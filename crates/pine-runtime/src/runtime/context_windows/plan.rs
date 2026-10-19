//! Preserve source request order while removing result computations that do
//! not reach an observable result. Expression state stays per call site.
use std::collections::{HashMap, HashSet};

use pine_ir::{
    CallSiteId, HirCallArg, HirExpr, HirExprKind, HirHistoryOffset, HirProgram, HirStmtKind,
    SymbolId,
};

use crate::builtins::args::call_arg_expr;
use crate::runtime::{call_plan::CallFamily, hir_walk};

#[derive(Clone, Debug)]
pub(crate) struct ContextCall {
    pub(crate) id: CallSiteId,
    pub(crate) callee: String,
    pub(crate) args: Vec<HirCallArg>,
}

#[derive(Debug, Default)]
pub(crate) struct ContextPlan {
    pub(crate) calls: Vec<ContextCall>,
    pub(crate) live: HashSet<CallSiteId>,
    pub(crate) root_live: HashSet<CallSiteId>,
    pub(crate) chart_passthrough: HashSet<CallSiteId>,
    pub(crate) initializers: HashMap<SymbolId, HirExpr>,
    pub(crate) guards: HashMap<CallSiteId, Vec<(HirExpr, bool)>>,
    pub(crate) writers: HashMap<SymbolId, Vec<HirExpr>>,
}

impl ContextPlan {
    pub(crate) fn from_program(program: &HirProgram) -> Self {
        let mut plan = Self::default();
        let mut roots = Vec::new();
        hir_walk::statements_and_bindings(
            &program.statements,
            &mut |expr| {
                if let HirExprKind::Call {
                    callee,
                    call_site_id,
                    args,
                } = &expr.kind
                    && matches!(
                        callee.as_str(),
                        "request.security" | "request.security_lower_tf"
                    )
                {
                    plan.calls.push(ContextCall {
                        id: *call_site_id,
                        callee: callee.clone(),
                        args: args.clone(),
                    });
                }
            },
            &mut |_| {},
        );
        if plan.calls.is_empty()
            || (program.calc_bars_count.unwrap_or(0) == 0
                && !plan.calls.iter().any(|call| {
                    call_arg_expr(
                        &call.args,
                        if call.callee == "request.security" {
                            7
                        } else {
                            6
                        },
                        "calc_bars_count",
                    )
                    .is_some()
                }))
        {
            return Self::default();
        }
        hir_walk::statements(&program.statements, &mut |expr| {
            if let HirExprKind::Call { callee, .. } = &expr.kind
                && matches!(
                    CallFamily::for_name(callee),
                    CallFamily::Output
                        | CallFamily::Drawing
                        | CallFamily::Alert
                        | CallFamily::Strategy
                        | CallFamily::RuntimeError
                )
            {
                roots.push(expr.clone());
            }
        });
        hir_walk::statements_and_bindings(&program.statements, &mut |_| {}, &mut |statement| {
            match &statement.kind {
                HirStmtKind::Decl { symbol, value }
                | HirStmtKind::Reassign { symbol, value }
                | HirStmtKind::FieldReassign { symbol, value, .. } => {
                    plan.writers.entry(*symbol).or_default().push(value.clone());
                }
                HirStmtKind::TupleDecl { symbols, value } => {
                    for symbol in symbols {
                        plan.writers.entry(*symbol).or_default().push(value.clone());
                    }
                }
                _ => {}
            }
        });
        hir_walk::statements_and_bindings(&program.statements, &mut |_| {}, &mut |statement| {
            if let HirStmtKind::Decl { symbol, value } = &statement.kind {
                plan.initializers
                    .entry(*symbol)
                    .or_insert_with(|| value.clone());
            }
        });
        plan.guards = super::guards::collect(&program.statements);
        if program.dynamic_requests && program.language_version.is_some_and(|v| v >= 6) {
            // Native distinguishes a pure input/simple guard from a const or
            // series guard. With simple selectors, the former preserves the
            // existing chart dataset, including its declaration-level bound.
            for call in &plan.calls {
                let Some(guards) = plan.guards.get(&call.id) else {
                    continue;
                };
                if guards
                    .iter()
                    .any(|(guard, _)| guard.pine_type.qualifier == pine_ir::Qualifier::Series)
                {
                    continue;
                }
                let guarded = guards.iter().any(|(guard, _)| {
                    matches!(
                        guard.pine_type.qualifier,
                        pine_ir::Qualifier::Input | pine_ir::Qualifier::Simple
                    )
                });
                let simple = [(0, "symbol"), (1, "timeframe")]
                    .into_iter()
                    .all(|(index, name)| {
                        call_arg_expr(&call.args, index, name).is_some_and(|arg| {
                            arg.pine_type.qualifier != pine_ir::Qualifier::Series
                        })
                    });
                if guarded && simple {
                    plan.chart_passthrough.insert(call.id);
                }
            }
        }
        super::dependencies::extend(program, &mut plan.writers);
        for root in roots.clone() {
            if let HirExprKind::Call { call_site_id, .. } = root.kind {
                roots.extend(
                    plan.guards
                        .get(&call_site_id)
                        .into_iter()
                        .flatten()
                        .map(|(guard, _)| guard.clone()),
                );
            }
        }
        let mut visited = HashSet::new();
        let mut pending = roots.clone();
        while let Some(expr) = pending.pop() {
            mark_expression(
                &expr,
                &mut plan.live,
                &mut visited,
                &mut pending,
                &plan.writers,
                true,
            );
        }
        visited.clear();
        pending = roots;
        while let Some(expr) = pending.pop() {
            mark_expression(
                &expr,
                &mut plan.root_live,
                &mut visited,
                &mut pending,
                &plan.writers,
                !program.dynamic_requests,
            );
        }
        plan
    }

    pub(crate) fn has_window_demands(&self) -> bool {
        !self.calls.is_empty()
    }
}

fn mark_expression(
    expr: &HirExpr,
    live: &mut HashSet<CallSiteId>,
    visited: &mut HashSet<SymbolId>,
    pending: &mut Vec<HirExpr>,
    writers: &HashMap<SymbolId, Vec<HirExpr>>,
    requested_calculations: bool,
) {
    match &expr.kind {
        HirExprKind::Symbol(symbol) => {
            if visited.insert(*symbol)
                && let Some(values) = writers.get(symbol)
            {
                pending.extend(values.iter().cloned());
            }
        }
        HirExprKind::Block { result, .. } => {
            // The symbol graph reaches only needed locals. Traversing every
            // block statement here would resurrect an unused inner request.
            mark_expression(
                result,
                live,
                visited,
                pending,
                writers,
                requested_calculations,
            );
        }
        HirExprKind::Call {
            callee,
            call_site_id,
            args,
        } => {
            if matches!(
                callee.as_str(),
                "request.security" | "request.security_lower_tf"
            ) {
                live.insert(*call_site_id);
            }
            let calculation = call_arg_expr(args, 2, "expression");
            for arg in args {
                if !requested_calculations
                    && matches!(
                        callee.as_str(),
                        "request.security" | "request.security_lower_tf"
                    )
                    && calculation.is_some_and(|value| std::ptr::eq(value, &arg.value))
                {
                    continue;
                }
                mark_expression(
                    &arg.value,
                    live,
                    visited,
                    pending,
                    writers,
                    requested_calculations,
                );
            }
        }
        HirExprKind::Unary { expr, .. } | HirExprKind::FieldAccess { value: expr, .. } => {
            pending.push((**expr).clone())
        }
        HirExprKind::History { expr, offset } => {
            pending.push((**expr).clone());
            if let HirHistoryOffset::Dynamic(offset) = offset {
                pending.push((**offset).clone());
            }
        }
        HirExprKind::Binary { left, right, .. } => {
            pending.extend([(**left).clone(), (**right).clone()])
        }
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            pending.extend([
                (**condition).clone(),
                (**then_expr).clone(),
                (**else_expr).clone(),
            ]);
        }
        HirExprKind::Tuple(values)
        | HirExprKind::UserTypeConstruct { fields: values, .. }
        | HirExprKind::UserTypeArrayConstruct {
            elements: values, ..
        } => pending.extend(values.iter().cloned()),
        HirExprKind::Switch { selector, arms } => {
            if let Some(selector) = selector {
                pending.push((**selector).clone());
            }
            for arm in arms {
                pending.extend(arm.condition.iter().cloned());
                pending.push(arm.result.clone());
            }
        }
        HirExprKind::For {
            from,
            to,
            step,
            result,
            ..
        } => {
            pending.extend([(**from).clone(), (**to).clone(), (**result).clone()]);
            pending.extend(step.iter().map(|step| (**step).clone()));
        }
        HirExprKind::ForIn {
            iterable, result, ..
        } => pending.extend([(**iterable).clone(), (**result).clone()]),
        HirExprKind::While {
            condition, result, ..
        } => pending.extend([(**condition).clone(), (**result).clone()]),
        HirExprKind::Literal(_) | HirExprKind::Builtin(_) => {}
    }
}
