//! Conservative source dependencies for direct-context reference admission.
//! This is not an initializer cache: prior writes must remain visible even
//! after a type/constant cache entry has been invalidated by reassignment.
use super::*;

impl Analyzer {
    pub(crate) fn record_request_reference_statement(&mut self, statement: &Stmt) {
        if self.legacy.dialect().version() < 5 {
            return;
        }
        let mut writes = Vec::new();
        let mut expressions = Vec::new();
        match &statement.kind {
            StmtKind::Decl { name, value, .. } | StmtKind::Reassign { name, value } => {
                if let Some(symbol) = self.scope.resolve(name)
                    && reference_kind(symbol.pine_type.kind)
                {
                    writes.push((symbol.id, value.clone()));
                    // Reference graphs can share nested values. Keep reverse
                    // dependencies conservatively; copy-independent graphs may
                    // remain unsupported rather than hiding an unsafe source.
                    let alias = Expr {
                        kind: ExprKind::Identifier(name.clone()),
                        span: statement.span,
                    };
                    for dependency in self.request_reference_roots(value) {
                        if dependency != symbol.id {
                            writes.push((dependency, alias.clone()));
                        }
                    }
                }
                expressions.push(value);
            }
            StmtKind::FieldReassign {
                receiver, value, ..
            } => {
                if let Some(symbol) = self.scope.resolve(receiver)
                    && reference_kind(symbol.pine_type.kind)
                {
                    writes.push((symbol.id, value.clone()));
                }
                expressions.push(value);
            }
            StmtKind::ArrayFieldReassign {
                array,
                index,
                value,
                ..
            } => {
                for symbol in self.request_reference_roots(array) {
                    writes.push((symbol, value.clone()));
                    writes.push((symbol, index.clone()));
                }
                expressions.extend([array, index, value]);
            }
            StmtKind::Expr(expr) => expressions.push(expr),
            // Branch statements have already visited their individual bodies.
            // Function declarations do not execute their bodies here.
            _ => {}
        }
        for expression in expressions {
            crate::modules::visit_expression(expression, &mut |expr| {
                let ExprKind::Call { callee, args } = &expr.kind else {
                    return;
                };
                let Some(name) = expr_name(callee) else {
                    return;
                };
                let collection = name.split('.').next().is_some_and(|prefix| {
                    matches!(prefix, "array" | "map" | "matrix")
                        || self.scope.resolve(prefix).is_some_and(|symbol| {
                            symbol.pine_type.kind.array_element_kind().is_some()
                                || symbol.pine_type.kind == ValueKind::Map
                                || crate::types::is_matrix_kind(symbol.pine_type.kind)
                        })
                });
                if !collection && !self.functions.contains_key(&name) {
                    return;
                }
                let mut roots = self.request_reference_roots(callee);
                for arg in args {
                    roots.extend(self.request_reference_roots(&arg.value));
                }
                // A no-argument UDF can mutate a global reference too.
                if let Some(group) = self.functions.get(&name) {
                    for function in std::iter::once(group).chain(group.overloads.iter()) {
                        let body = match &function.body {
                            FunctionBody::Expr(value) => value.clone(),
                            FunctionBody::Block(body) => Expr {
                                kind: ExprKind::If {
                                    condition: Box::new(Expr {
                                        kind: ExprKind::Literal(Literal::Bool(true)),
                                        span: expr.span,
                                    }),
                                    then_branch: body.clone(),
                                    else_branch: Vec::new(),
                                },
                                span: expr.span,
                            },
                        };
                        roots.extend(
                            self.with_source_context_ref(function.source_context_id, |analyzer| {
                                analyzer.request_reference_roots(&body)
                            }),
                        );
                    }
                }
                roots.sort_unstable_by_key(|symbol| symbol.0);
                roots.dedup();
                writes.extend(roots.into_iter().map(|symbol| (symbol, expr.clone())));
            });
        }
        let context = self.current_source_context_id();
        for (symbol, expr) in writes {
            let sources = self.request_reference_sources.entry(symbol).or_default();
            if !sources
                .iter()
                .any(|source| source.source_context_id == context && source.expr.span == expr.span)
            {
                sources.push(SourcedExpr {
                    source_context_id: context,
                    expr,
                });
            }
        }
    }

    fn request_reference_roots(&self, expr: &Expr) -> Vec<SymbolId> {
        let mut roots = Vec::new();
        crate::modules::visit_expression(expr, &mut |expr| {
            let name = match &expr.kind {
                ExprKind::Identifier(name) => name.as_str(),
                ExprKind::QualifiedName(parts) => parts.first().map_or("", String::as_str),
                _ => return,
            };
            if let Some(symbol) = self
                .bound_symbol(name, expr.span)
                .or_else(|| self.scope.resolve(name))
                && reference_kind(symbol.pine_type.kind)
            {
                roots.push(symbol.id);
            }
        });
        roots
    }
}
