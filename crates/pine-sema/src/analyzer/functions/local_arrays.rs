use std::collections::HashSet;

use crate::prelude::*;

/// Recognize straight-line mutations of arrays allocated by this function.
/// Unknown control flow discards the proof; aliases and parameters do not
/// establish ownership. Argument expressions still undergo normal validation.
pub(crate) fn local_array_mutation_spans(body: &FunctionBody) -> Vec<Span> {
    let FunctionBody::Block(statements) = body else {
        return Vec::new();
    };
    let mut owned = HashSet::new();
    let mut spans = Vec::new();
    for statement in statements {
        let straight = match &statement.kind {
            StmtKind::Decl { value, .. }
            | StmtKind::Reassign { value, .. }
            | StmtKind::TupleDecl { value, .. }
            | StmtKind::Expr(value) => straight_expr(value),
            _ => false,
        };
        if !straight {
            owned.clear();
            continue;
        }
        match &statement.kind {
            StmtKind::Decl { name, value, .. } => {
                owned.remove(name);
                if let ExprKind::Call { callee, .. } = &value.kind
                    && expr_name(callee).is_some_and(|name| {
                        matches!(
                            name.as_str(),
                            "array.new<float>"
                                | "array.new_float"
                                | "array.new<int>"
                                | "array.new_int"
                                | "array.new<bool>"
                                | "array.new_bool"
                                | "array.new<string>"
                                | "array.new_string"
                                | "array.new<color>"
                                | "array.new_color"
                        )
                    })
                    && !statements.iter().any(|stmt| binding_may_change(stmt, name))
                {
                    owned.insert(name.clone());
                }
            }
            StmtKind::Reassign { name, .. } => {
                owned.remove(name);
            }
            StmtKind::Expr(Expr {
                kind: ExprKind::Call { callee, args },
                ..
            }) => {
                if expr_name(callee)
                    .is_some_and(|name| matches!(name.as_str(), "array.clear" | "array.push"))
                    && let Some(receiver) = args
                        .iter()
                        .find(|arg| arg.name.as_deref() == Some("id"))
                        .or_else(|| args.first().filter(|arg| arg.name.is_none()))
                    && let ExprKind::Identifier(name) = &receiver.value.kind
                    && owned.contains(name)
                {
                    spans.push(callee.span);
                }
            }
            StmtKind::TupleDecl { .. } => {}
            _ => owned.clear(),
        }
    }
    spans
}

// A `var` initializer does not run again on later bars. A reassignment even
// after the mutation may replace its receiver for the next execution.
fn binding_may_change(stmt: &Stmt, name: &str) -> bool {
    match &stmt.kind {
        StmtKind::Reassign {
            name: target,
            value,
        } => target == name || expr_may_rebind(value, name),
        StmtKind::Decl { value, .. }
        | StmtKind::TupleDecl { value, .. }
        | StmtKind::Expr(value) => expr_may_rebind(value, name),
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_may_rebind(condition, name)
                || then_branch
                    .iter()
                    .chain(else_branch)
                    .any(|stmt| binding_may_change(stmt, name))
        }
        // Unsupported statement forms cannot establish stable ownership.
        _ => true,
    }
}

fn expr_may_rebind(expr: &Expr, name: &str) -> bool {
    match &expr.kind {
        ExprKind::Literal(_) | ExprKind::Identifier(_) | ExprKind::QualifiedName(_) => false,
        ExprKind::Group(expr) | ExprKind::Unary { expr, .. } => expr_may_rebind(expr, name),
        ExprKind::Binary { left, right, .. } => {
            expr_may_rebind(left, name) || expr_may_rebind(right, name)
        }
        ExprKind::History { expr, offset } => {
            expr_may_rebind(expr, name) || expr_may_rebind(offset, name)
        }
        ExprKind::Call { callee, args } => {
            expr_may_rebind(callee, name)
                || args.iter().any(|arg| expr_may_rebind(&arg.value, name))
        }
        ExprKind::Tuple(items) => items.iter().any(|expr| expr_may_rebind(expr, name)),
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_may_rebind(condition, name)
                || expr_may_rebind(then_expr, name)
                || expr_may_rebind(else_expr, name)
        }
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_may_rebind(condition, name)
                || then_branch
                    .iter()
                    .chain(else_branch)
                    .any(|stmt| binding_may_change(stmt, name))
        }
        ExprKind::Switch { selector, arms } => {
            selector
                .as_deref()
                .is_some_and(|expr| expr_may_rebind(expr, name))
                || arms.iter().any(|arm| {
                    arm.condition
                        .as_ref()
                        .is_some_and(|expr| expr_may_rebind(expr, name))
                        || match &arm.result {
                            SwitchArmResult::Expr(expr) => expr_may_rebind(expr, name),
                            SwitchArmResult::Block(stmts) => {
                                stmts.iter().any(|stmt| binding_may_change(stmt, name))
                            }
                        }
                })
        }
        _ => true,
    }
}

fn straight_expr(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Literal(_) | ExprKind::Identifier(_) | ExprKind::QualifiedName(_) => true,
        ExprKind::Group(expr) | ExprKind::Unary { expr, .. } => straight_expr(expr),
        ExprKind::Binary { left, right, .. } => straight_expr(left) && straight_expr(right),
        ExprKind::History { expr, offset } => straight_expr(expr) && straight_expr(offset),
        ExprKind::Call { callee, args } => {
            straight_expr(callee) && args.iter().all(|arg| straight_expr(&arg.value))
        }
        ExprKind::Tuple(items) => items.iter().all(straight_expr),
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => straight_expr(condition) && straight_expr(then_expr) && straight_expr(else_expr),
        _ => false,
    }
}

impl Analyzer {
    pub(crate) fn allows_local_array_mutation(&self, span: Span) -> bool {
        self.legacy.dialect() >= crate::PineDialect::V5
            && !self
                .function_context_is_method
                .last()
                .copied()
                .unwrap_or(false)
            && self
                .function_stack
                .last()
                .and_then(|name| self.functions.get(name))
                .is_some_and(|function| {
                    std::iter::once(function)
                        .chain(function.overloads.iter())
                        .any(|candidate| {
                            local_array_mutation_spans(&candidate.body).contains(&span)
                        })
                })
    }
}
