//! Admission proof for plain scalar collection parameters and local aliases.
//! Complex object fields retain the separate owned-field capability boundary.
use super::*;
use std::collections::HashSet;

pub(super) fn parameter_mutation_spans(
    body: &FunctionBody,
    params: &[pine_syntax::FunctionParam],
    declarations: &[Stmt],
) -> Vec<Span> {
    let shadowed = declarations.iter().any(|s| {
        matches!(&s.kind,
        StmtKind::Import(import) if import.alias.as_ref().is_some_and(|a|matches!(a.name.as_str(), "array"|"matrix"|"map")))
    });
    if shadowed {
        return Vec::new();
    }
    let mut proof = Proof {
        known: HashSet::new(),
        locals: HashSet::new(),
        params: HashSet::new(),
        spans: Vec::new(),
        invalid: false,
    };
    for p in params {
        proof.locals.insert(p.name.clone());
        proof.params.insert(p.name.clone());
        if let Some(ty) = &p.type_name {
            let ty = ty
                .strip_prefix("simple ")
                .or_else(|| ty.strip_prefix("series "))
                .or_else(|| ty.strip_prefix("const "))
                .unwrap_or(ty);
            if crate::analyzer::functions::collections::collection_parameter_info(ty).is_some() {
                proof.known.insert(p.name.clone());
            }
        }
    }
    match body {
        FunctionBody::Block(b) => proof.block(b),
        FunctionBody::Expr(e) => proof.expr(e),
    }
    if proof.invalid {
        Vec::new()
    } else {
        proof.spans
    }
}

struct Proof {
    known: HashSet<String>,
    locals: HashSet<String>,
    params: HashSet<String>,
    spans: Vec<Span>,
    invalid: bool,
}

impl Proof {
    fn reference(&self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Identifier(n) => self.known.contains(n),
            ExprKind::Group(e) => self.reference(e),
            ExprKind::Call { callee, .. } => {
                crate::analyzer::calls::expr_name(callee).is_some_and(|n| {
                    let template = n.replacen(".new<", "<", 1);
                    (n.contains(".new<")
                        && crate::analyzer::functions::collections::collection_parameter_info(
                            &template,
                        )
                        .is_some())
                        || matches!(
                            n.as_str(),
                            "array.new_float"
                                | "array.new_int"
                                | "array.new_bool"
                                | "array.new_string"
                                | "array.new_color"
                        )
                })
            }
            ExprKind::Ternary {
                then_expr,
                else_expr,
                ..
            } => self.reference(then_expr) && self.reference(else_expr),
            _ => false,
        }
    }
    fn expr(&mut self, e: &Expr) {
        visit_expr(e, &mut |e| {
            let ExprKind::Call { callee, args } = &e.kind else {
                return;
            };
            let Some(name) = crate::analyzer::calls::expr_name(callee) else {
                self.invalid = true;
                return;
            };
            let parts: Vec<_> = name.split('.').collect();
            if parts.len() > 2 {
                self.invalid = true;
                return;
            }
            let namespace = parts.first().copied().unwrap_or("");
            let method = parts.get(1).copied().unwrap_or("");
            let builtin_receiver = matches!(namespace, "array" | "map" | "matrix");
            let mutation = crate::analyzer::calls::is_array_mutation_method_call_name(&name)
                || crate::analyzer::calls::is_map_mutation_method_call_name(&name);
            if mutation {
                let known = if builtin_receiver {
                    args.iter()
                        .find(|a| a.name.as_deref() == Some("id"))
                        .or_else(|| args.first().filter(|a| a.name.is_none()))
                        .is_some_and(|a| self.reference(&a.value))
                } else {
                    self.known.contains(namespace)
                };
                if known {
                    self.spans.push(callee.span);
                } else {
                    self.invalid = true;
                }
            } else if !builtin_receiver
                && !matches!(namespace, "math" | "str" | "color" | "input" | "table")
                && !self.known.contains(namespace)
                && !pine_builtins::is_phase_1_builtin(&name)
            {
                self.invalid = true;
            }
            if self.known.contains(namespace) && method.is_empty() {
                self.invalid = true;
            }
        });
    }
    fn nested(&mut self, b: &[Stmt]) {
        let known = self.known.clone();
        let locals = self.locals.clone();
        self.block(b);
        self.known.retain(|n| known.contains(n));
        self.locals = locals;
    }
    fn block(&mut self, b: &[Stmt]) {
        for s in b {
            match &s.kind {
                StmtKind::Decl { name, value, .. } => {
                    self.expr(value);
                    let reference = self.reference(value);
                    if self.params.contains(name) {
                        self.invalid = true;
                    }
                    self.locals.insert(name.clone());
                    self.known.remove(name);
                    if reference {
                        self.known.insert(name.clone());
                    }
                }
                StmtKind::Reassign { name, value } => {
                    self.expr(value);
                    let reference = self.reference(value);
                    if self.params.contains(name)
                        || !self.locals.contains(name)
                        || (self.known.contains(name) && !reference)
                    {
                        self.invalid = true;
                    }
                    self.known.remove(name);
                    if reference {
                        self.known.insert(name.clone());
                    }
                }
                StmtKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    self.expr(condition);
                    self.nested(then_branch);
                    self.nested(else_branch);
                }
                StmtKind::For {
                    from,
                    to,
                    step,
                    body,
                    ..
                } => {
                    self.expr(from);
                    self.expr(to);
                    if let Some(step) = step {
                        self.expr(step)
                    }
                    self.nested(body);
                }
                StmtKind::While { condition, body } => {
                    self.expr(condition);
                    self.nested(body);
                }
                StmtKind::ForIn { iterable, body, .. } => {
                    self.expr(iterable);
                    self.nested(body);
                }
                StmtKind::Expr(e) => self.expr(e),
                _ => {
                    self.invalid = true;
                }
            }
        }
    }
}
