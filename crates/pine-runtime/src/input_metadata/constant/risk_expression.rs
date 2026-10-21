//! Isolated scalar control flow for immutable risk arguments. The normal Pine
//! evaluator owns loop/assignment semantics; no original program scope executes.
use std::collections::HashSet;
use std::sync::Arc;

use pine_ir::{HirExpr, HirExprKind, HirStmtKind};

use super::{
    ConstEvaluator, ExecutionLimits, HistoricalRuntime, PineValue, literal_expr, pure_call,
};

pub(super) fn contextual_call(name: &str) -> bool {
    pine_builtins::is_pure_scalar_string_builtin(name)
        || name.starts_with("ticker.") && pine_builtins::is_phase_1_builtin(name)
        || matches!(
            name,
            "timestamp"
                | "math.round_to_mintick"
                | "timeframe.in_seconds"
                | "timeframe.from_seconds"
                | "syminfo.prefix"
                | "syminfo.ticker"
        )
}

impl ConstEvaluator<'_> {
    pub(super) fn risk_scalar_flow(&mut self, expr: &HirExpr, depth: usize) -> Option<PineValue> {
        let mut locals = HashSet::new();
        let mut statement_locals = HashSet::new();
        crate::runtime::hir_walk::expression(
            expr,
            &mut |node| {
                if let HirExprKind::For { counter, .. } = &node.kind {
                    locals.insert(*counter);
                }
            },
            &mut |statement| match &statement.kind {
                HirStmtKind::Decl { symbol, .. } => {
                    statement_locals.insert(*symbol);
                }
                HirStmtKind::TupleDecl { symbols, .. } => {
                    statement_locals.extend(symbols);
                }
                HirStmtKind::For { counter, .. } => {
                    statement_locals.insert(*counter);
                }
                _ => {}
            },
        );
        locals.extend(statement_locals);
        let mut safe = true;
        let mut statements_safe = true;
        let mut free = Vec::new();
        crate::runtime::hir_walk::expression(
            expr,
            &mut |node| match &node.kind {
                HirExprKind::Symbol(symbol) if !locals.contains(symbol) => free.push(node),
                HirExprKind::Builtin(name) => {
                    safe &= node.pine_type.qualifier != pine_ir::Qualifier::Series
                        && (name == "na"
                            || name.starts_with("syminfo.")
                            || name.starts_with("timeframe.")
                            || !matches!(
                                crate::builtins::variables::eval_static_builtin_value(name),
                                PineValue::Void
                            ));
                }
                HirExprKind::Call { callee, args, .. } => {
                    safe &= pure_call(callee, args)
                        || contextual_call(callee)
                        || callee == "input"
                        || callee.starts_with("input.");
                }
                HirExprKind::Literal(_)
                | HirExprKind::Symbol(_)
                | HirExprKind::Unary { .. }
                | HirExprKind::Binary { .. }
                | HirExprKind::Ternary { .. }
                | HirExprKind::Switch { .. }
                | HirExprKind::Block { .. }
                | HirExprKind::For { .. }
                | HirExprKind::While { .. }
                | HirExprKind::Tuple(_) => {}
                _ => safe = false,
            },
            &mut |statement| {
                statements_safe &= match &statement.kind {
                    HirStmtKind::Reassign { symbol, .. } => locals.contains(symbol),
                    HirStmtKind::Expr(_)
                    | HirStmtKind::Decl { .. }
                    | HirStmtKind::TupleDecl { .. }
                    | HirStmtKind::If { .. }
                    | HirStmtKind::Switch { .. }
                    | HirStmtKind::For { .. }
                    | HirStmtKind::While { .. }
                    | HirStmtKind::Break
                    | HirStmtKind::Continue => true,
                    _ => false,
                };
            },
        );
        if !safe || !statements_safe {
            return None;
        }
        let mut expression = expr.clone();
        let mut values = Vec::new();
        for node in free {
            if let HirExprKind::Symbol(symbol) = node.kind {
                let value = self.eval(node, depth)?;
                // Seed only scalar values, never handles into another runtime.
                literal_expr(node, value.clone())?;
                values.push((symbol, value));
            }
        }
        let (overrides, environment) = self.risk_context?;
        let mut program = self.minimal.clone();
        program.symbols = self.program.symbols.clone();
        program.next_series_id = self.program.next_series_id;
        program.next_var_slot_id = self.program.next_var_slot_id;
        let prepared = crate::PreparedProgram::from_shared_hir(Arc::new(program));
        let mut runtime =
            HistoricalRuntime::from_prepared_with_request_environment_and_input_overrides(
                &prepared,
                environment.clone(),
                overrides.clone(),
            )
            .with_execution_limits(ExecutionLimits {
                max_steps_per_bar: self.remaining,
                max_loop_iterations_per_bar: self.max_loop_iterations,
            });
        for (symbol, value) in values {
            runtime.set_symbol_value(symbol, value);
        }
        // Runtime result bookkeeping is unnecessary for declaration evaluation.
        expression.series_id = None;
        runtime.reset_execution_budget();
        let value = runtime.eval_expr(&expression).ok();
        self.remaining = runtime.execution_steps_remaining;
        value
    }
}
