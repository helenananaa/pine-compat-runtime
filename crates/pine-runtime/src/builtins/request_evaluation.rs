//! Requested-expression replay and immutable dependency reconstruction.
use super::{request_values::RequestedValue, requests::collect_hir_expr_symbols};
use crate::{Bar, BarUpdateKind, HistoricalRuntime, PineValue, RequestEnvironment, RuntimeError};
use pine_ir::{HirExpr, HirStmt, HirStmtKind, Qualifier, SymbolId, ValueKind};
use std::collections::{HashMap, HashSet};

impl HistoricalRuntime<'_> {
    pub(crate) fn fork_with_request_environment(
        &self,
        request_environment: RequestEnvironment,
    ) -> Self {
        let mut runtime = Self::with_runtime_program(self.program.clone(), request_environment);
        runtime.input_overrides = self.input_overrides.clone();
        runtime.request_feed = self.request_feed.clone();
        runtime.static_requests = self.static_requests.for_requested_context();
        runtime.context_windows = self.context_windows.clone();
        runtime.request_window_history = self.request_window_history.clone();
        let chart = runtime.request_environment.chart();
        runtime.context_windows.enter_scope(crate::RequestKey::new(
            chart.symbol(),
            chart.timeframe().clone(),
        ));
        runtime.inherit_execution_budget(self);
        runtime
    }

    pub(crate) fn eval_requested_bar_expression(
        &mut self,
        bar: Bar,
        expression: &HirExpr,
        captures: &HashMap<SymbolId, PineValue>,
        dependency_initializers: &HashMap<SymbolId, &HirExpr>,
        tuple_dependencies: &HashMap<SymbolId, &HirStmt>,
    ) -> Result<RequestedValue, RuntimeError> {
        self.eval_requested_bar_expression_with_kind(
            bar,
            expression,
            captures,
            dependency_initializers,
            tuple_dependencies,
            BarUpdateKind::Historical,
        )
    }

    pub(crate) fn eval_requested_bar_expression_with_kind(
        &mut self,
        bar: Bar,
        expression: &HirExpr,
        captures: &HashMap<SymbolId, PineValue>,
        dependency_initializers: &HashMap<SymbolId, &HirExpr>,
        tuple_dependencies: &HashMap<SymbolId, &HirStmt>,
        update_kind: BarUpdateKind,
    ) -> Result<RequestedValue, RuntimeError> {
        let bar_index = self.bars;
        self.current_bar_update_kind = update_kind;
        self.current_bar_is_new = true;
        self.current_bar = Some(bar);
        self.series_store.set_current_bar(bar_index);
        self.current_symbols.clear();
        self.current_series.clear();
        self.set_builtin_symbols(&bar, bar_index)?;
        self.current_symbols.extend(
            captures
                .iter()
                .map(|(symbol, value)| (*symbol, value.clone())),
        );
        self.eval_requested_expression_dependencies(
            expression,
            &mut HashSet::new(),
            dependency_initializers,
            tuple_dependencies,
        )?;

        let value = self.eval_expr(expression)?;
        let value = self.freeze_requested_value(&value)?;
        self.snapshot_persistent_array_aliases()?;
        self.commit_current_series()?;
        self.previous_bar_time = Some(bar.time);
        self.bars += 1;
        self.collect_temporary_collections();
        self.current_bar_update_kind = BarUpdateKind::Historical;
        self.current_bar_is_new = true;
        self.current_bar = None;
        Ok(value)
    }

    fn eval_requested_expression_dependencies(
        &mut self,
        expression: &HirExpr,
        visiting: &mut HashSet<SymbolId>,
        dependency_initializers: &HashMap<SymbolId, &HirExpr>,
        tuple_dependencies: &HashMap<SymbolId, &HirStmt>,
    ) -> Result<(), RuntimeError> {
        let mut symbols = Vec::new();
        collect_hir_expr_symbols(expression, &mut symbols);
        for symbol in symbols {
            self.eval_requested_symbol_dependency(
                symbol,
                visiting,
                dependency_initializers,
                tuple_dependencies,
            )?;
        }
        Ok(())
    }

    fn eval_requested_symbol_dependency(
        &mut self,
        symbol: SymbolId,
        visiting: &mut HashSet<SymbolId>,
        dependency_initializers: &HashMap<SymbolId, &HirExpr>,
        tuple_dependencies: &HashMap<SymbolId, &HirStmt>,
    ) -> Result<(), RuntimeError> {
        if self.current_symbols.contains_key(&symbol) {
            return Ok(());
        }
        let pine_type = self
            .program
            .symbols
            .iter()
            .find(|candidate| candidate.id == symbol)
            .map(|candidate| candidate.pine_type)
            .ok_or_else(|| RuntimeError {
                message: format!(
                    "request.security expression references unknown symbol {}",
                    symbol.0
                ),
            })?;
        if pine_type.kind == ValueKind::Na {
            return Ok(());
        }
        if !matches!(pine_type.qualifier, Qualifier::Series | Qualifier::Simple) {
            return Err(RuntimeError {
                message: format!(
                    "request.security expression capture {} was not initialized before the request",
                    symbol.0
                ),
            });
        }
        if !visiting.insert(symbol) {
            // Analysis admits a recursive provider dependency only through a
            // positive history reference. Its current value will be computed
            // after the historical read, from this requested context's series.
            return Ok(());
        }
        if let Some(initializer) = dependency_initializers.get(&symbol).copied() {
            self.eval_requested_expression_dependencies(
                initializer,
                visiting,
                dependency_initializers,
                tuple_dependencies,
            )?;
            let value = self.eval_expr(initializer)?;
            self.set_symbol_value(symbol, value);
        } else if let Some(statement) = tuple_dependencies.get(&symbol).copied() {
            let HirStmtKind::TupleDecl { value, .. } = &statement.kind else {
                unreachable!("tuple dependency index contains only tuple declarations")
            };
            self.eval_requested_expression_dependencies(
                value,
                visiting,
                dependency_initializers,
                tuple_dependencies,
            )?;
            self.eval_stmt(statement)?;
        } else {
            return Err(RuntimeError {
                message: format!(
                    "request.security expression symbol {} is not an immutable admitted dependency",
                    symbol.0
                ),
            });
        }
        visiting.remove(&symbol);
        Ok(())
    }
}
