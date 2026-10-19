//! Attach surrounding conditions without evaluating series guards during the
//! static context phase. Only context-independent guards can remove a demand.
use crate::runtime::hir_walk;
use pine_ir::{CallSiteId, HirExpr, HirExprKind, HirStmt, HirStmtKind};
use std::collections::HashMap;

type Guards = HashMap<CallSiteId, Vec<(HirExpr, bool)>>;

pub(super) fn collect(statements: &[HirStmt]) -> Guards {
    let constraints = std::cell::RefCell::new(Vec::new());
    hir_walk::statements_and_bindings(
        statements,
        &mut |expr| {
            if let HirExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } = &expr.kind
            {
                constraints.borrow_mut().push((
                    calls_in_expr(then_expr),
                    (**condition).clone(),
                    true,
                ));
                constraints.borrow_mut().push((
                    calls_in_expr(else_expr),
                    (**condition).clone(),
                    false,
                ));
            }
        },
        &mut |stmt| {
            if let HirStmtKind::If {
                condition,
                then_branch,
                else_branch,
            } = &stmt.kind
            {
                constraints.borrow_mut().push((
                    calls_in_statements(then_branch),
                    condition.clone(),
                    true,
                ));
                constraints.borrow_mut().push((
                    calls_in_statements(else_branch),
                    condition.clone(),
                    false,
                ));
            }
        },
    );
    let mut guards = Guards::new();
    for (calls, condition, expected) in constraints.into_inner() {
        for call in calls {
            guards
                .entry(call)
                .or_default()
                .push((condition.clone(), expected));
        }
    }
    guards
}

fn calls_in_expr(expr: &HirExpr) -> Vec<CallSiteId> {
    let mut calls = Vec::new();
    hir_walk::expression(
        expr,
        &mut |expr| {
            if let HirExprKind::Call { call_site_id, .. } = expr.kind {
                calls.push(call_site_id);
            }
        },
        &mut |_| {},
    );
    calls
}

fn calls_in_statements(statements: &[HirStmt]) -> Vec<CallSiteId> {
    let mut calls = Vec::new();
    hir_walk::statements(statements, &mut |expr| {
        if let HirExprKind::Call { call_site_id, .. } = expr.kind {
            calls.push(call_site_id);
        }
    });
    calls
}
