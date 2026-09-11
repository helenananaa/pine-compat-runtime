use pine_ir::{HirExpr, HirExprKind, HirHistoryOffset, HirStmt, HirStmtKind};

pub(super) fn statements(stmts: &[HirStmt], visitor: &mut impl FnMut(&HirExpr)) {
    for statement in stmts {
        match &statement.kind {
            HirStmtKind::Expr(expr)
            | HirStmtKind::Decl { value: expr, .. }
            | HirStmtKind::Reassign { value: expr, .. }
            | HirStmtKind::FieldReassign { value: expr, .. }
            | HirStmtKind::TupleDecl { value: expr, .. } => {
                expression(expr, visitor);
            }
            HirStmtKind::ArrayFieldReassign {
                array,
                index,
                value,
                ..
            } => {
                expression(array, visitor);
                expression(index, visitor);
                expression(value, visitor);
            }
            HirStmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                expression(condition, visitor);
                statements(then_branch, visitor);
                statements(else_branch, visitor);
            }
            HirStmtKind::Switch { selector, arms } => {
                if let Some(selector) = selector {
                    expression(selector, visitor);
                }
                for arm in arms {
                    if let Some(condition) = &arm.condition {
                        expression(condition, visitor);
                    }
                    statements(&arm.body, visitor);
                }
            }
            HirStmtKind::For {
                from,
                to,
                step,
                body,
                ..
            } => {
                expression(from, visitor);
                expression(to, visitor);
                if let Some(step) = step {
                    expression(step, visitor);
                }
                statements(body, visitor);
            }
            HirStmtKind::ForIn { iterable, body, .. } => {
                expression(iterable, visitor);
                statements(body, visitor);
            }
            HirStmtKind::While { condition, body } => {
                expression(condition, visitor);
                statements(body, visitor);
            }
            HirStmtKind::Break | HirStmtKind::Continue => {}
        }
    }
}

fn expression(expr: &HirExpr, visitor: &mut impl FnMut(&HirExpr)) {
    visitor(expr);
    match &expr.kind {
        HirExprKind::Call { args, .. } => {
            for arg in args {
                expression(&arg.value, visitor);
            }
        }
        HirExprKind::Unary { expr, .. } | HirExprKind::FieldAccess { value: expr, .. } => {
            expression(expr, visitor)
        }
        HirExprKind::History { expr, offset } => {
            expression(expr, visitor);
            if let HirHistoryOffset::Dynamic(offset) = offset {
                expression(offset, visitor);
            }
        }
        HirExprKind::Binary { left, right, .. } => {
            expression(left, visitor);
            expression(right, visitor);
        }
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            expression(condition, visitor);
            expression(then_expr, visitor);
            expression(else_expr, visitor);
        }
        HirExprKind::Switch { selector, arms } => {
            if let Some(selector) = selector {
                expression(selector, visitor);
            }
            for arm in arms {
                if let Some(condition) = &arm.condition {
                    expression(condition, visitor);
                }
                expression(&arm.result, visitor);
            }
        }
        HirExprKind::For {
            from,
            to,
            step,
            statements: body,
            result,
            ..
        } => {
            expression(from, visitor);
            expression(to, visitor);
            if let Some(step) = step {
                expression(step, visitor);
            }
            statements(body, visitor);
            expression(result, visitor);
        }
        HirExprKind::ForIn {
            iterable,
            statements: body,
            result,
            ..
        } => {
            expression(iterable, visitor);
            statements(body, visitor);
            expression(result, visitor);
        }
        HirExprKind::While {
            condition,
            statements: body,
            result,
        } => {
            expression(condition, visitor);
            statements(body, visitor);
            expression(result, visitor);
        }
        HirExprKind::Tuple(values)
        | HirExprKind::UserTypeConstruct { fields: values, .. }
        | HirExprKind::UserTypeArrayConstruct {
            elements: values, ..
        } => {
            for value in values {
                expression(value, visitor);
            }
        }
        HirExprKind::Block {
            statements: body,
            result,
        } => {
            statements(body, visitor);
            expression(result, visitor);
        }
        HirExprKind::Literal(_) | HirExprKind::Symbol(_) | HirExprKind::Builtin(_) => {}
    }
}
