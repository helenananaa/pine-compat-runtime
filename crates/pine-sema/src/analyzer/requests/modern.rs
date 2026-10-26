use super::*;
use std::collections::HashSet;

mod references;
use references::RequestScope;

/// State of one request admission query.
#[derive(Default)]
struct RequestWalk {
    // Symbols on the current dependency path; rejects cyclic initializers.
    visiting: HashSet<SymbolId>,
    calls: Vec<String>,
    // Shared initializers and reference sources are reached on many paths.
    // A check that hit no cycle cut (a back edge or a call-stack guard) does
    // not depend on its path, so its answer is reused for the same symbol
    // and reference mode at the same or a shallower call depth.
    checked: HashMap<(SymbolId, bool), (bool, usize)>,
    cuts: usize,
}

#[cfg(test)]
thread_local! {
    static SYMBOL_CHECKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl Analyzer {
    pub(super) fn modern_request_expression_supported(&self, expr: &Expr) -> bool {
        self.modern_request_expr(expr, &RequestScope::default(), &mut RequestWalk::default())
    }

    pub(super) fn modern_request_reference_expression_supported(&self, expr: &Expr) -> bool {
        self.modern_request_expression_supported(expr)
            || self.modern_request_expr(
                expr,
                &RequestScope::references(),
                &mut RequestWalk::default(),
            )
    }

    /// Checks a symbol's dependencies once per query. `back_edge` answers a
    /// symbol that is already on the current path.
    fn modern_request_symbol(
        &self,
        key: (SymbolId, bool),
        back_edge: bool,
        walk: &mut RequestWalk,
        check: impl FnOnce(&Self, &mut RequestWalk) -> bool,
    ) -> bool {
        let (symbol, _) = key;
        if walk.visiting.contains(&symbol) {
            walk.cuts += 1;
            return back_edge;
        }
        if let Some(&(valid, depth)) = walk.checked.get(&key)
            && walk.calls.len() <= depth
        {
            return valid;
        }
        #[cfg(test)]
        SYMBOL_CHECKS.with(|checks| checks.set(checks.get() + 1));
        let cuts = walk.cuts;
        walk.visiting.insert(symbol);
        let valid = check(self, walk);
        walk.visiting.remove(&symbol);
        if walk.cuts == cuts {
            walk.checked.insert(key, (valid, walk.calls.len()));
        }
        valid
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
        walk: &mut RequestWalk,
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
                        return self.modern_request_symbol(
                            (symbol.id, true),
                            true,
                            walk,
                            |analyzer, walk| {
                                sources.iter().all(|source| {
                                    analyzer.with_source_context_ref(
                                        source.source_context_id,
                                        |analyzer| {
                                            analyzer.modern_request_expr(
                                                &source.expr,
                                                &RequestScope::references(),
                                                walk,
                                            )
                                        },
                                    )
                                })
                            },
                        );
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
                    self.modern_request_symbol(
                        (symbol.id, locals.references),
                        false,
                        walk,
                        |analyzer, walk| {
                            analyzer.with_source_context_ref(
                                initializer.source_context_id,
                                |analyzer| {
                                    analyzer.modern_request_expr(
                                        &initializer.expr,
                                        &RequestScope::with_references(locals.references),
                                        walk,
                                    )
                                },
                            )
                        },
                    )
                } else {
                    is_request_provider_scalar_name(name) || name == "na"
                }
            }
            ExprKind::Member { receiver, .. } if locals.references => {
                self.modern_request_expr(receiver, locals, walk)
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
                        walk,
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
                self.modern_request_expr(value, locals, walk)
            }
            ExprKind::History { expr, offset } => {
                self.modern_request_expr(expr, locals, walk)
                    && self.modern_request_expr(offset, locals, walk)
            }
            ExprKind::Binary { left, right, .. } => {
                self.modern_request_expr(left, locals, walk)
                    && self.modern_request_expr(right, locals, walk)
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => [condition, then_expr, else_expr]
                .into_iter()
                .all(|expr| self.modern_request_expr(expr, locals, walk)),
            ExprKind::Tuple(items) => items
                .iter()
                .all(|expr| self.modern_request_expr(expr, locals, walk)),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.modern_request_expr(condition, locals, walk)
                    && self.modern_request_block(then_branch, locals, walk)
                    && self.modern_request_block(else_branch, locals, walk)
            }
            ExprKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => self.modern_request_for((counter, from, to, step.as_deref()), body, locals, walk),
            ExprKind::While { condition, body } => {
                self.modern_request_expr(condition, locals, walk)
                    && self.modern_request_block(body, locals, walk)
            }
            ExprKind::Switch { selector, arms } => {
                selector
                    .as_deref()
                    .is_none_or(|selector| self.modern_request_expr(selector, locals, walk))
                    && arms.iter().all(|arm| {
                        arm.condition.as_ref().is_none_or(|condition| {
                            self.modern_request_expr(condition, locals, walk)
                        }) && match &arm.result {
                            SwitchArmResult::Expr(expr) => {
                                self.modern_request_expr(expr, locals, walk)
                            }
                            SwitchArmResult::Block(body) => {
                                self.modern_request_block(body, locals, walk)
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
                    if walk.calls.len() >= 64 || walk.calls.contains(&key) {
                        walk.cuts += 1;
                        return false;
                    }
                    let Ok(args) = function.complete_args(args, expr.span) else {
                        return false;
                    };
                    if !args
                        .iter()
                        .all(|arg| self.modern_request_expr(&arg.value, locals, walk))
                    {
                        return false;
                    }
                    walk.calls.push(key);
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
                                    analyzer.modern_request_expr(expr, &params, walk)
                                }
                                FunctionBody::Block(body) => {
                                    analyzer.modern_request_block(body, &params, walk)
                                }
                            }
                        });
                    walk.calls.pop();
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
                            self.modern_request_expr(&expression.value, locals, walk)
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
                        && self.modern_request_expr(&args[2].value, locals, walk);
                }
                if locals.references
                    && self.reference_request_call_supported(callee, expr, args, locals, walk)
                {
                    return true;
                }
                let source_default = super::source_args::source_input_default(args);
                // Input metadata is checked by the ordinary call analyzer.
                // Only defval participates in the requested value graph; the
                // runtime inherits overrides using this call site's identity.
                if request_input_call_is_supported(&name) {
                    return source_default
                        .is_some_and(|value| self.modern_request_expr(value, locals, walk));
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
                            && self.modern_request_expr(&arg.value, locals, walk)
                    });
                }
                (array_constructor
                    || request_scalar_call_is_supported(&name)
                    || request_tuple_call_is_supported(&name)
                    || matches!(name.as_str(), "timeframe.change" | "year"))
                    && args
                        .iter()
                        .all(|arg| self.modern_request_expr(&arg.value, locals, walk))
            }
            _ => false,
        }
    }

    fn modern_request_block(
        &self,
        body: &[Stmt],
        locals: &RequestScope,
        walk: &mut RequestWalk,
    ) -> bool {
        let mut locals = locals.clone();
        for statement in body {
            let valid = match &statement.kind {
                StmtKind::Decl { name, value, .. } => {
                    let valid = self.modern_request_expr(value, &locals, walk);
                    let ty = self.type_of_expr_with_params(value, &locals.types);
                    locals.insert_typed(name.clone(), ty);
                    valid
                }
                StmtKind::Reassign { name, value } => {
                    locals.contains(name) && self.modern_request_expr(value, &locals, walk)
                }
                StmtKind::TupleDecl { names, value } => {
                    let valid = self.modern_request_expr(value, &locals, walk);
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
                        walk,
                    ) && self.modern_request_expr(value, &locals, walk)
                }
                StmtKind::ArrayFieldReassign {
                    array,
                    index,
                    value,
                    ..
                } if locals.references => {
                    self.modern_request_expr(array, &locals, walk)
                        && self.modern_request_expr(index, &locals, walk)
                        && self.modern_request_expr(value, &locals, walk)
                }
                StmtKind::Expr(expr) => self.modern_request_expr(expr, &locals, walk),
                StmtKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    self.modern_request_expr(condition, &locals, walk)
                        && self.modern_request_block(then_branch, &locals, walk)
                        && self.modern_request_block(else_branch, &locals, walk)
                }
                StmtKind::For {
                    counter,
                    from,
                    to,
                    step,
                    body,
                } => {
                    self.modern_request_for((counter, from, to, step.as_ref()), body, &locals, walk)
                }
                StmtKind::While { condition, body } => {
                    self.modern_request_expr(condition, &locals, walk)
                        && self.modern_request_block(body, &locals, walk)
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
        walk: &mut RequestWalk,
    ) -> bool {
        let (counter, from, to, step) = bounds;
        if !self.modern_request_expr(from, locals, walk)
            || !self.modern_request_expr(to, locals, walk)
            || step.is_some_and(|step| !self.modern_request_expr(step, locals, walk))
        {
            return false;
        }
        let mut loop_locals = locals.clone();
        loop_locals.insert(counter.to_owned());
        self.modern_request_block(body, &loop_locals, walk)
    }
}

#[cfg(test)]
mod tests {
    use super::SYMBOL_CHECKS;
    use pine_syntax::SourceFile;

    /// `x{i} = x{i-1} + x{i-1}` doubles the paths to `x0` at every level.
    fn shared_dependency_graph(prelude: &str, root: &str, depth: usize, request: &str) -> String {
        let mut source =
            format!("//@version=6\nindicator(\"shared dependencies\")\n{prelude}x0 = {root}\n");
        for level in 1..=depth {
            let previous = level - 1;
            source.push_str(&format!("x{level} = x{previous} + x{previous}\n"));
        }
        let request = request.replace("{last}", &format!("x{depth}"));
        source.push_str(&format!(
            "plot(request.security(\"OTHER\", \"1\", {request}))\n"
        ));
        source
    }

    fn analyze_counting_checks(source: &str) -> (crate::Analysis, usize) {
        SYMBOL_CHECKS.with(|checks| checks.set(0));
        let analysis = crate::analyze_source(&SourceFile::new("shared-dependencies.pine", source));
        (analysis, SYMBOL_CHECKS.with(std::cell::Cell::get))
    }

    fn diagnostic_codes(analysis: &crate::Analysis) -> Vec<&str> {
        analysis
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect()
    }

    #[test]
    fn request_admission_checks_each_shared_initializer_once_per_query() {
        const DEPTH: usize = 16;
        let source = shared_dependency_graph("", "close", DEPTH, "{last}");
        let (analysis, checks) = analyze_counting_checks(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
        // One admission query checks each initializer once. The per-path
        // walk made 2^(DEPTH + 1) - 1 = 131,071 checks at this depth.
        assert!(
            checks <= 2 * (DEPTH + 1),
            "{checks} initializer checks for depth {DEPTH}"
        );
    }

    #[test]
    fn unsupported_dependency_behind_a_shared_graph_is_still_rejected() {
        let mutable = "var float total = 0.0\ntotal += close\n";
        let clean = shared_dependency_graph(mutable, "close", 12, "{last}");
        let (analysis, _) = analyze_counting_checks(&clean);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        // The left operand checks the whole shared graph first; the mutable
        // operand on the right must still be rejected by the same query.
        let after = shared_dependency_graph(mutable, "close", 12, "{last} + total");
        let (analysis, _) = analyze_counting_checks(&after);
        assert_eq!(
            diagnostic_codes(&analysis),
            vec!["E_UNSUPPORTED_FEATURE"],
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());

        // A rejected root is reused on every path, and still rejects.
        let shared = shared_dependency_graph(mutable, "total", 12, "{last}");
        let (analysis, _) = analyze_counting_checks(&shared);
        assert_eq!(
            diagnostic_codes(&analysis),
            vec!["E_UNSUPPORTED_FEATURE"],
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }
}
