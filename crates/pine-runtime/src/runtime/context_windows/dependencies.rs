//! Observable mutations and control dependencies participate in request-result
//! liveness, including writes through a directly aliased reference binding.
use crate::runtime::hir_walk;
use pine_ir::{HirExpr, HirExprKind, HirProgram, HirStmt, HirStmtKind, SymbolId, ValueKind};
use std::collections::HashMap;

pub(super) fn extend(program: &HirProgram, writers: &mut HashMap<SymbolId, Vec<HirExpr>>) {
    hir_walk::statements_and_bindings(
        &program.statements,
        &mut |expr| {
            if let HirExprKind::Call { callee, args, .. } = &expr.kind
                && mutator(callee)
                && let Some(receiver) = args.first()
                && let HirExprKind::Symbol(symbol) = receiver.value.kind
            {
                writers.entry(symbol).or_default().push(expr.clone());
            }
        },
        &mut |_| {},
    );
    hir_walk::statements_and_bindings(&program.statements, &mut |_| {}, &mut |statement| {
        match &statement.kind {
            HirStmtKind::Decl { symbol, value } | HirStmtKind::Reassign { symbol, value }
                if reference(value.pine_type.kind) =>
            {
                if let HirExprKind::Symbol(source) = value.kind {
                    let mut reverse = value.clone();
                    reverse.kind = HirExprKind::Symbol(*symbol);
                    writers.entry(source).or_default().push(reverse);
                }
            }
            HirStmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                control_writes(then_branch, condition, writers);
                control_writes(else_branch, condition, writers);
            }
            HirStmtKind::While { condition, body } => control_writes(body, condition, writers),
            HirStmtKind::For {
                from,
                to,
                step,
                body,
                ..
            } => {
                for expr in [Some(from), Some(to), step.as_ref()].into_iter().flatten() {
                    control_writes(body, expr, writers);
                }
            }
            HirStmtKind::ForIn { iterable, body, .. } => control_writes(body, iterable, writers),
            HirStmtKind::Switch { selector, arms } => {
                for arm in arms {
                    for condition in selector.iter().chain(arm.condition.iter()) {
                        control_writes(&arm.body, condition, writers);
                    }
                }
            }
            _ => {}
        }
    });
}

fn control_writes(
    statements: &[HirStmt],
    condition: &HirExpr,
    writers: &mut HashMap<SymbolId, Vec<HirExpr>>,
) {
    hir_walk::statements_and_bindings(statements, &mut |_| {}, &mut |statement| match &statement
        .kind
    {
        HirStmtKind::Decl { symbol, .. }
        | HirStmtKind::Reassign { symbol, .. }
        | HirStmtKind::FieldReassign { symbol, .. } => {
            writers.entry(*symbol).or_default().push(condition.clone())
        }
        HirStmtKind::TupleDecl { symbols, .. } => {
            for symbol in symbols {
                writers.entry(*symbol).or_default().push(condition.clone());
            }
        }
        _ => {}
    });
}

fn reference(kind: ValueKind) -> bool {
    !matches!(
        kind,
        ValueKind::Int
            | ValueKind::Float
            | ValueKind::Bool
            | ValueKind::String
            | ValueKind::Color
            | ValueKind::Na
            | ValueKind::Void
            | ValueKind::Tuple
            | ValueKind::PlotDisplay
            | ValueKind::StrategyDirection
    )
}

fn mutator(name: &str) -> bool {
    let Some((namespace, method)) = name.split_once('.') else {
        return false;
    };
    matches!(namespace, "array" | "map" | "matrix")
        && matches!(
            method,
            "set"
                | "push"
                | "pop"
                | "shift"
                | "unshift"
                | "insert"
                | "remove"
                | "clear"
                | "fill"
                | "sort"
                | "reverse"
                | "swap"
                | "concat"
                | "put"
                | "put_all"
                | "add_row"
                | "add_col"
                | "remove_row"
                | "remove_col"
                | "swap_rows"
                | "swap_columns"
        )
}
