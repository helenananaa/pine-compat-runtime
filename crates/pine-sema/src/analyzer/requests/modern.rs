use super::*;
use std::collections::HashSet;

mod references;
use references::RequestScope;

impl Analyzer {
    pub(super) fn modern_request_expression_supported(&self, expr: &Expr) -> bool {
        self.modern_request_expr(
            expr,
            &RequestScope::default(),
            &mut HashSet::new(),
            &mut Vec::new(),
        )
    }

    pub(super) fn modern_request_reference_expression_supported(&self, expr: &Expr) -> bool {
        self.modern_request_expression_supported(expr)
            || self.modern_request_expr(
                expr,
                &RequestScope::references(),
                &mut HashSet::new(),
                &mut Vec::new(),
            )
    }

    fn modern_request_binding_is_reassigned(&self, name: &str, symbol_id: SymbolId) -> bool {
        if self.scope.symbol_is_global(symbol_id) {
            return self.request_global_reassigned_names.contains(name);
        }
        if !self.request_reassigned_names.contains(name) {
            return false;
        }
        // The global prepass records names from every UDF. For a local
        // binding, a helper UDF's reassignment of the same spelling does not
        // mutate this binding. Restrict the check to the active function body.
        let Some(function) = self
            .function_stack
            .last()
            .and_then(|name| self.functions.get(name))
        else {
            return true;
        };
        let mut reassigned = HashSet::new();
        crate::analyzer::statements::collect_request_reassigned_names_from_function_body(
            &function.body,
            &mut reassigned,
        );
        for overload in &function.overloads {
            crate::analyzer::statements::collect_request_reassigned_names_from_function_body(
                &overload.body,
                &mut reassigned,
            );
        }
        reassigned.contains(name)
    }

    fn modern_request_expr(
        &self,
        expr: &Expr,
        locals: &RequestScope,
        visiting: &mut HashSet<SymbolId>,
        calls: &mut Vec<String>,
    ) -> bool {
        match &expr.kind {
            ExprKind::Literal(_) => true,
            ExprKind::Identifier(name) => {
                if locals.contains(name) {
                    return true;
                }
                if let Some(symbol) = self
                    .bound_symbol(name, expr.span)
                    .or_else(|| self.scope.resolve(name))
                {
                    if crate::symbols::initial_symbol(name)
                        .is_some_and(|builtin| builtin.id == symbol.id)
                    {
                        return true;
                    }
                    let reference =
                        locals.references && references::reference_kind(symbol.pine_type.kind);
                    // External persistent/mutable scalar variables cannot be reconstructed
                    // from their initializer alone. Function-local state is evaluated
                    // inside the requested block and takes the locals path above.
                    if !reference && self.modern_request_binding_is_reassigned(name, symbol.id) {
                        return false;
                    }
                    if reference
                        && let Some(sources) = self.request_reference_sources.get(&symbol.id)
                    {
                        // Reference aliases and prior in-place updates can form
                        // cycles. The function call stack still rejects recursive
                        // UDFs; each reference dependency is checked once here.
                        if !visiting.insert(symbol.id) {
                            return true;
                        }
                        let valid = sources.iter().all(|source| {
                            self.with_source_context_ref(source.source_context_id, |analyzer| {
                                analyzer.modern_request_expr(
                                    &source.expr,
                                    &RequestScope::references(),
                                    visiting,
                                    calls,
                                )
                            })
                        });
                        visiting.remove(&symbol.id);
                        return valid;
                    }
                    let Some(initializer) = self
                        .symbol_init_exprs
                        .get(&symbol.id)
                        .or_else(|| {
                            (self.legacy.dialect().version() >= 5)
                                .then(|| self.request_param_sources.get(&symbol.id))
                                .flatten()
                        })
                        .or_else(|| {
                            self.symbol_tuple_value_sources
                                .get(&symbol.id)
                                .map(|(source, _)| source)
                        })
                    else {
                        return false;
                    };
                    if !is_request_scalar_type(symbol.pine_type) && !reference {
                        return false;
                    }
                    if !reference && symbol.pine_type.qualifier != Qualifier::Series {
                        return true;
                    }
                    if !reference && symbol.persistence != PersistenceKind::None {
                        return false;
                    }
                    if !visiting.insert(symbol.id) {
                        return false;
                    }
                    let valid =
                        self.with_source_context_ref(initializer.source_context_id, |analyzer| {
                            analyzer.modern_request_expr(
                                &initializer.expr,
                                &RequestScope::with_references(locals.references),
                                visiting,
                                calls,
                            )
                        });
                    visiting.remove(&symbol.id);
                    valid
                } else {
                    is_request_provider_scalar_name(name) || name == "na"
                }
            }
            ExprKind::Member { receiver, .. } if locals.references => {
                self.modern_request_expr(receiver, locals, visiting, calls)
            }
            ExprKind::QualifiedName(parts)
                if locals.references
                    && parts.len() > 1
                    && self.modern_request_expr(
                        &Expr {
                            kind: ExprKind::Identifier(parts[0].clone()),
                            span: expr.span,
                        },
                        locals,
                        visiting,
                        calls,
                    ) =>
            {
                true
            }
            ExprKind::QualifiedName(_) => expr_name(expr).is_some_and(|name| {
                is_request_provider_scalar_name(&name)
                    || matches!(
                        name.as_str(),
                        "syminfo.timezone"
                            | "timeframe.isticks"
                            | "timeframe.period"
                            | "timeframe.main_period"
                    )
                    || crate::types::const_color_value(expr).is_some()
                    || crate::types::const_numeric_value(expr).is_some()
            }),
            ExprKind::Group(value) | ExprKind::Unary { expr: value, .. } => {
                self.modern_request_expr(value, locals, visiting, calls)
            }
            ExprKind::History { expr, offset } => {
                self.modern_request_expr(expr, locals, visiting, calls)
                    && self.modern_request_expr(offset, locals, visiting, calls)
            }
            ExprKind::Binary { left, right, .. } => {
                self.modern_request_expr(left, locals, visiting, calls)
                    && self.modern_request_expr(right, locals, visiting, calls)
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => [condition, then_expr, else_expr]
                .into_iter()
                .all(|expr| self.modern_request_expr(expr, locals, visiting, calls)),
            ExprKind::Tuple(items) => items
                .iter()
                .all(|expr| self.modern_request_expr(expr, locals, visiting, calls)),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.modern_request_expr(condition, locals, visiting, calls)
                    && self.modern_request_block(then_branch, locals, visiting, calls)
                    && self.modern_request_block(else_branch, locals, visiting, calls)
            }
            ExprKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => self.modern_request_for(
                (counter, from, to, step.as_deref()),
                body,
                locals,
                visiting,
                calls,
            ),
            ExprKind::While { condition, body } => {
                self.modern_request_expr(condition, locals, visiting, calls)
                    && self.modern_request_block(body, locals, visiting, calls)
            }
            ExprKind::Switch { selector, arms } => {
                selector.as_deref().is_none_or(|selector| {
                    self.modern_request_expr(selector, locals, visiting, calls)
                }) && arms.iter().all(|arm| {
                    arm.condition.as_ref().is_none_or(|condition| {
                        self.modern_request_expr(condition, locals, visiting, calls)
                    }) && match &arm.result {
                        SwitchArmResult::Expr(expr) => {
                            self.modern_request_expr(expr, locals, visiting, calls)
                        }
                        SwitchArmResult::Block(body) => {
                            self.modern_request_block(body, locals, visiting, calls)
                        }
                    }
                })
            }
            ExprKind::Call { callee, args } => {
                let Some(name) = self.request_expression_call_name(callee) else {
                    return false;
                };
                if let Some(group) = self.functions.get(&name) {
                    let types: Vec<_> = args
                        .iter()
                        .map(|arg| self.type_of_expr_with_params(&arg.value, &locals.types))
                        .collect();
                    let function = if locals.references {
                        let Some(function) = group.select_overload(args, &types) else {
                            return false;
                        };
                        function
                    } else {
                        group
                    };
                    let key = format!("{:?}:{name}", function.source_context_id);
                    if calls.len() >= 64 || calls.contains(&key) {
                        return false;
                    }
                    let Ok(args) = function.complete_args(args, expr.span) else {
                        return false;
                    };
                    if !args
                        .iter()
                        .all(|arg| self.modern_request_expr(&arg.value, locals, visiting, calls))
                    {
                        return false;
                    }
                    calls.push(key);
                    let mut params = RequestScope::with_references(locals.references);
                    for (i, (name, arg)) in function.params.iter().zip(args.iter()).enumerate() {
                        let ty = self
                            .type_of_expr_with_params(&arg.value, &locals.types)
                            .or_else(|| {
                                function
                                    .param_types
                                    .get(i)
                                    .and_then(|p| p.as_ref().map(|p| p.pine_type))
                            });
                        params.insert_typed(name.clone(), ty);
                    }
                    let valid =
                        self.with_source_context_ref(function.source_context_id, |analyzer| {
                            match &function.body {
                                FunctionBody::Expr(expr) => {
                                    analyzer.modern_request_expr(expr, &params, visiting, calls)
                                }
                                FunctionBody::Block(body) => {
                                    analyzer.modern_request_block(body, &params, visiting, calls)
                                }
                            }
                        });
                    calls.pop();
                    return valid;
                }
                if name == "request.security" {
                    // Keep nested requests on the independently admitted scalar
                    // path until reference prerequisites can be replayed there.
                    if locals.references {
                        return false;
                    }
                    let Ok(args) =
                        super::source_args::normalize_security_source_args(args, expr.span)
                    else {
                        return false;
                    };
                    if !self.dynamic_requests {
                        return args.get(2).is_some_and(|expression| {
                            self.modern_request_expr(&expression.value, locals, visiting, calls)
                        });
                    }
                    return self.dynamic_requests
                        && (matches!(
                            &args[0].value.without_groups().kind,
                            ExprKind::Literal(Literal::String(_))
                        ) || expr_name(&args[0].value).as_deref() == Some("syminfo.tickerid"))
                        && (matches!(
                            &args[1].value.without_groups().kind,
                            ExprKind::Literal(Literal::String(_))
                        ) || matches!(
                            expr_name(&args[1].value).as_deref(),
                            Some("timeframe.main_period" | "timeframe.period")
                        ))
                        && self.modern_request_expr(&args[2].value, locals, visiting, calls);
                }
                if locals.references
                    && self.reference_request_call_supported(
                        callee, expr, args, locals, visiting, calls,
                    )
                {
                    return true;
                }
                let source_default = super::source_args::source_input_default(args);
                // Input metadata is checked by the ordinary call analyzer.
                // Only defval participates in the requested value graph; the
                // runtime inherits overrides using this call site's identity.
                if request_input_call_is_supported(&name) {
                    return source_default.is_some_and(|value| {
                        self.modern_request_expr(value, locals, visiting, calls)
                    });
                }
                let array_constructor = matches!(
                    name.as_str(),
                    "array.from"
                        | "array.new_float"
                        | "array.new_int"
                        | "array.new_bool"
                        | "array.new_string"
                        | "array.new_color"
                        | "array.new<float>"
                        | "array.new<int>"
                        | "array.new<bool>"
                        | "array.new<string>"
                        | "array.new<color>"
                        | "ta.pivot_point_levels"
                );
                if pine_builtins::is_pure_scalar_string_builtin(&name) {
                    return args.iter().all(|arg| {
                        self.type_of_expr_with_params(&arg.value, &HashMap::new())
                            .is_some_and(is_request_scalar_type)
                            && self.modern_request_expr(&arg.value, locals, visiting, calls)
                    });
                }
                (array_constructor
                    || request_scalar_call_is_supported(&name)
                    || request_tuple_call_is_supported(&name)
                    || matches!(name.as_str(), "timeframe.change" | "year"))
                    && args
                        .iter()
                        .all(|arg| self.modern_request_expr(&arg.value, locals, visiting, calls))
            }
            _ => false,
        }
    }

    fn modern_request_block(
        &self,
        body: &[Stmt],
        locals: &RequestScope,
        visiting: &mut HashSet<SymbolId>,
        calls: &mut Vec<String>,
    ) -> bool {
        let mut locals = locals.clone();
        for statement in body {
            let valid = match &statement.kind {
                StmtKind::Decl { name, value, .. } => {
                    let valid = self.modern_request_expr(value, &locals, visiting, calls);
                    let ty = self.type_of_expr_with_params(value, &locals.types);
                    locals.insert_typed(name.clone(), ty);
                    valid
                }
                StmtKind::Reassign { name, value } => {
                    locals.contains(name)
                        && self.modern_request_expr(value, &locals, visiting, calls)
                }
                StmtKind::TupleDecl { names, value } => {
                    let valid = self.modern_request_expr(value, &locals, visiting, calls);
                    locals.extend(names.iter().cloned());
                    valid
                }
                StmtKind::FieldReassign {
                    receiver, value, ..
                } if locals.references => {
                    self.modern_request_expr(
                        &Expr {
                            kind: ExprKind::Identifier(receiver.clone()),
                            span: statement.span,
                        },
                        &locals,
                        visiting,
                        calls,
                    ) && self.modern_request_expr(value, &locals, visiting, calls)
                }
                StmtKind::ArrayFieldReassign {
                    array,
                    index,
                    value,
                    ..
                } if locals.references => {
                    self.modern_request_expr(array, &locals, visiting, calls)
                        && self.modern_request_expr(index, &locals, visiting, calls)
                        && self.modern_request_expr(value, &locals, visiting, calls)
                }
                StmtKind::Expr(expr) => self.modern_request_expr(expr, &locals, visiting, calls),
                StmtKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    self.modern_request_expr(condition, &locals, visiting, calls)
                        && self.modern_request_block(then_branch, &locals, visiting, calls)
                        && self.modern_request_block(else_branch, &locals, visiting, calls)
                }
                StmtKind::For {
                    counter,
                    from,
                    to,
                    step,
                    body,
                } => self.modern_request_for(
                    (counter, from, to, step.as_ref()),
                    body,
                    &locals,
                    visiting,
                    calls,
                ),
                StmtKind::While { condition, body } => {
                    self.modern_request_expr(condition, &locals, visiting, calls)
                        && self.modern_request_block(body, &locals, visiting, calls)
                }
                StmtKind::Break | StmtKind::Continue => true,
                _ => false,
            };
            if !valid {
                return false;
            }
        }
        true
    }

    fn modern_request_for(
        &self,
        bounds: (&str, &Expr, &Expr, Option<&Expr>),
        body: &[Stmt],
        locals: &RequestScope,
        visiting: &mut HashSet<SymbolId>,
        calls: &mut Vec<String>,
    ) -> bool {
        let (counter, from, to, step) = bounds;
        if !self.modern_request_expr(from, locals, visiting, calls)
            || !self.modern_request_expr(to, locals, visiting, calls)
            || step.is_some_and(|step| !self.modern_request_expr(step, locals, visiting, calls))
        {
            return false;
        }
        let mut loop_locals = locals.clone();
        loop_locals.insert(counter.to_owned());
        self.modern_request_block(body, &loop_locals, visiting, calls)
    }
}
