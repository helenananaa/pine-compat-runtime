use std::collections::{HashMap, HashSet};

use pine_ir::{
    CallSiteId, HirCallArg, HirExpr, HirExprKind, HirHistoryOffset, HirStmt, HirStmtKind,
    Qualifier, SymbolId, ValueKind,
};

use crate::builtins::args::call_arg_expr;
use crate::builtins::time::calendar_timeframe_close;
use crate::runtime::append_history::AppendHistory;
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestGaps {
    Off,
    On,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestLookahead {
    Off,
    On,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RequestMergePolicy {
    gaps: RequestGaps,
    lookahead: RequestLookahead,
}

impl RequestMergePolicy {
    const MODERN: Self = Self {
        gaps: RequestGaps::Off,
        lookahead: RequestLookahead::Off,
    };
}

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_request_call(
        &mut self,
        callee: &str,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Option<Result<PineValue, RuntimeError>> {
        let (merge, legacy) = match callee {
            "request.security" => (RequestMergePolicy::MODERN, None),
            "$legacy.security.gaps_off.lookahead_off" => (
                RequestMergePolicy {
                    gaps: RequestGaps::Off,
                    lookahead: RequestLookahead::Off,
                },
                legacy_source_span(args),
            ),
            "$legacy.security.gaps_on.lookahead_off" => (
                RequestMergePolicy {
                    gaps: RequestGaps::On,
                    lookahead: RequestLookahead::Off,
                },
                legacy_source_span(args),
            ),
            "$legacy.security.gaps_off.lookahead_on" => (
                RequestMergePolicy {
                    gaps: RequestGaps::Off,
                    lookahead: RequestLookahead::On,
                },
                legacy_source_span(args),
            ),
            "$legacy.security.gaps_on.lookahead_on" => (
                RequestMergePolicy {
                    gaps: RequestGaps::On,
                    lookahead: RequestLookahead::On,
                },
                legacy_source_span(args),
            ),
            _ => return None,
        };
        if merge.lookahead == RequestLookahead::On {
            self.legacy_security_repaint_warnings
                .entry(call_site_id)
                .or_insert(legacy.unwrap_or((0, 0)));
        }
        let result = self.eval_request_security(call_site_id, args, merge);
        Some(match legacy {
            Some((start, end)) => result.map_err(|error| RuntimeError {
                message: format!(
                    "legacy security at source span {start}..{end}: {}",
                    error.message
                ),
            }),
            None => result,
        })
    }

    fn eval_request_security(
        &mut self,
        call_site_id: CallSiteId,
        args: &[HirCallArg],
        merge: RequestMergePolicy,
    ) -> Result<PineValue, RuntimeError> {
        if !(3..=5).contains(&args.len()) {
            return Err(RuntimeError {
                message: format!(
                    "request.security expects 3 to 5 argument(s), got {}",
                    args.len()
                ),
            });
        }

        let Some(symbol_expr) = call_arg_expr(args, 0, "symbol") else {
            return Err(RuntimeError {
                message: "request.security missing symbol argument".to_owned(),
            });
        };
        let PineValue::String(symbol) = self.eval_expr(symbol_expr)? else {
            return Err(RuntimeError {
                message: "request.security symbol must evaluate to string".to_owned(),
            });
        };
        let Some(timeframe_expr) = call_arg_expr(args, 1, "timeframe") else {
            return Err(RuntimeError {
                message: "request.security missing timeframe argument".to_owned(),
            });
        };
        let PineValue::String(timeframe) = self.eval_expr(timeframe_expr)? else {
            return Err(RuntimeError {
                message: "request.security timeframe must evaluate to string".to_owned(),
            });
        };
        let Some(expression) = call_arg_expr(args, 2, "expression") else {
            return Err(RuntimeError {
                message: "request.security missing expression argument".to_owned(),
            });
        };

        let requested_timeframe =
            RequestTimeframe::parse(&timeframe).map_err(|err| RuntimeError {
                message: err.to_string(),
            })?;
        let chart = self.request_environment.chart();
        let chart_symbol = chart.symbol().to_owned();
        let chart_timeframe = chart.timeframe().clone();

        if symbol == chart_symbol && requested_timeframe == chart_timeframe {
            return self.eval_expr(expression);
        }

        self.eval_provider_security(
            call_site_id,
            &symbol,
            requested_timeframe,
            &chart_timeframe,
            expression,
            merge,
        )
    }

    fn eval_provider_security(
        &mut self,
        call_site_id: CallSiteId,
        symbol: &str,
        requested_timeframe: RequestTimeframe,
        chart_timeframe: &RequestTimeframe,
        expression: &HirExpr,
        merge: RequestMergePolicy,
    ) -> Result<PineValue, RuntimeError> {
        validate_provider_timeframe(symbol, &requested_timeframe, chart_timeframe)?;
        let key = RequestKey::new(symbol, requested_timeframe.clone());
        let current_time = self
            .current_bar
            .map(|bar| bar.time)
            .ok_or_else(|| RuntimeError {
                message: "request.security has no current chart bar".to_owned(),
            })?;

        let cache_key = RequestCacheKey::new(call_site_id, key.symbol(), key.timeframe().value());
        let include_forming = self.current_bar_update_kind == BarUpdateKind::Forming
            && self.request_feed.has_forming(&key);
        if include_forming || !self.request_cache.contains_key(&cache_key) {
            let requested_chart = if key.symbol() == self.request_environment.chart().symbol() {
                self.request_environment
                    .chart()
                    .clone()
                    .with_timeframe(requested_timeframe.clone())
            } else {
                ChartContext::new(key.symbol(), requested_timeframe.clone())
            };
            let requested_environment = self.request_environment.for_chart(requested_chart);
            let requested_values = self.evaluate_request_incremental(
                &key,
                &cache_key,
                expression,
                requested_environment,
                include_forming,
            )?;
            let aligned = align_requested_value(
                &requested_values,
                current_time,
                &requested_timeframe,
                chart_timeframe,
                merge,
                self.current_bar_update_kind,
            );
            if !include_forming {
                self.request_cache
                    .insert(cache_key.clone(), requested_values);
            }
            return Ok(aligned);
        }

        let requested_values = self
            .request_cache
            .get(&cache_key)
            .ok_or_else(|| RuntimeError {
                message: "request.security requested context cache was not populated".to_owned(),
            })?;
        Ok(align_requested_value(
            requested_values,
            current_time,
            &requested_timeframe,
            chart_timeframe,
            merge,
            self.current_bar_update_kind,
        ))
    }

    pub(crate) fn resolved_request_bars_from(
        &self,
        key: &RequestKey,
        start: usize,
    ) -> Result<Vec<Bar>, RuntimeError> {
        let provider_bars = match self.request_environment.provider().bars(key) {
            Ok(bars) => bars,
            Err(RequestDataError::MissingData { symbol, timeframe }) => {
                if !self.request_feed.contains(key) {
                    return Err(RuntimeError {
                        message: RequestDataError::MissingData { symbol, timeframe }.to_string(),
                    });
                }
                &[]
            }
            Err(error) => {
                return Err(RuntimeError {
                    message: error.to_string(),
                });
            }
        };
        let include_forming = self.current_bar_update_kind == BarUpdateKind::Forming;
        let bars = self
            .request_feed
            .resolved_from(key, provider_bars, include_forming, start);
        if bars.is_empty() {
            return Err(RuntimeError {
                message: RequestDataError::MissingData {
                    symbol: key.symbol().to_owned(),
                    timeframe: key.timeframe().value().to_owned(),
                }
                .to_string(),
            });
        }
        Ok(bars)
    }

    pub(crate) fn evaluate_requested_values(
        &mut self,
        requested_bars: &[Bar],
        expression: &HirExpr,
        requested_environment: RequestEnvironment,
    ) -> Result<Vec<(i64, PineValue)>, RuntimeError> {
        let dependency_initializers = request_dependency_initializers(&self.program);
        let captures = request_capture_values(
            &self.program,
            expression,
            &self.current_symbols,
            &dependency_initializers,
        );
        let mut runtime = self.fork_with_request_environment(requested_environment);
        runtime.historical_end = Some(requested_bars.len());
        let mut values = Vec::with_capacity(requested_bars.len());
        for bar in requested_bars {
            values.push((
                bar.time,
                runtime.eval_requested_bar_expression(
                    *bar,
                    expression,
                    &captures,
                    &dependency_initializers,
                )?,
            ));
        }
        self.legacy_security_repaint_warnings
            .extend(runtime.legacy_security_repaint_warnings);
        Ok(values)
    }

    pub(crate) fn eval_requested_bar_expression(
        &mut self,
        bar: Bar,
        expression: &HirExpr,
        captures: &HashMap<SymbolId, PineValue>,
        dependency_initializers: &HashMap<SymbolId, &HirExpr>,
    ) -> Result<PineValue, RuntimeError> {
        let bar_index = self.bars;
        self.current_bar_update_kind = BarUpdateKind::Historical;
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
        )?;

        let value = self.eval_expr(expression)?;
        self.commit_current_series()?;
        self.previous_bar_time = Some(bar.time);
        self.bars += 1;
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
    ) -> Result<(), RuntimeError> {
        let mut symbols = Vec::new();
        collect_hir_expr_symbols(expression, &mut symbols);
        for symbol in symbols {
            self.eval_requested_symbol_dependency(symbol, visiting, dependency_initializers)?;
        }
        Ok(())
    }

    fn eval_requested_symbol_dependency(
        &mut self,
        symbol: SymbolId,
        visiting: &mut HashSet<SymbolId>,
        dependency_initializers: &HashMap<SymbolId, &HirExpr>,
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
        if pine_type.qualifier != Qualifier::Series {
            return Err(RuntimeError {
                message: format!(
                    "request.security expression capture {} was not initialized before the request",
                    symbol.0
                ),
            });
        }
        if !visiting.insert(symbol) {
            return Err(RuntimeError {
                message: format!(
                    "request.security expression has a cyclic immutable alias dependency at symbol {}",
                    symbol.0
                ),
            });
        }
        let initializer = dependency_initializers
            .get(&symbol)
            .copied()
            .cloned()
            .ok_or_else(|| RuntimeError {
                message: format!(
                    "request.security expression symbol {} is not an immutable admitted dependency",
                    symbol.0
                ),
            })?;
        self.eval_requested_expression_dependencies(
            &initializer,
            visiting,
            dependency_initializers,
        )?;
        let value = self.eval_expr(&initializer)?;
        self.set_symbol_value(symbol, value);
        visiting.remove(&symbol);
        Ok(())
    }
}

pub(crate) fn request_capture_values(
    program: &pine_ir::HirProgram,
    expression: &HirExpr,
    current_symbols: &HashMap<SymbolId, PineValue>,
    dependency_initializers: &HashMap<SymbolId, &HirExpr>,
) -> HashMap<SymbolId, PineValue> {
    let mut candidates = HashSet::new();
    collect_request_capture_symbols(
        program,
        expression,
        &mut HashSet::new(),
        &mut candidates,
        dependency_initializers,
    );
    candidates
        .into_iter()
        .filter_map(|symbol| {
            current_symbols
                .get(&symbol)
                .cloned()
                .map(|value| (symbol, value))
        })
        .collect()
}

fn collect_request_capture_symbols(
    program: &pine_ir::HirProgram,
    expression: &HirExpr,
    visited: &mut HashSet<SymbolId>,
    captures: &mut HashSet<SymbolId>,
    dependency_initializers: &HashMap<SymbolId, &HirExpr>,
) {
    let mut symbols = Vec::new();
    collect_hir_expr_symbols(expression, &mut symbols);
    for symbol in symbols {
        if !visited.insert(symbol) {
            continue;
        }
        let Some(pine_type) = program
            .symbols
            .iter()
            .find(|candidate| candidate.id == symbol)
            .map(|candidate| candidate.pine_type)
        else {
            continue;
        };
        let Some(initializer) = dependency_initializers.get(&symbol).copied() else {
            continue;
        };
        if pine_type.qualifier == Qualifier::Series {
            collect_request_capture_symbols(
                program,
                initializer,
                visited,
                captures,
                dependency_initializers,
            );
        } else {
            captures.insert(symbol);
        }
    }
}

pub(crate) fn request_dependency_initializers(
    program: &pine_ir::HirProgram,
) -> HashMap<SymbolId, &HirExpr> {
    let mut initializers = HashMap::new();
    collect_request_stmt_initializers(&program.statements, &mut initializers);
    initializers
}

fn collect_request_stmt_initializers<'a>(
    statements: &'a [HirStmt],
    initializers: &mut HashMap<SymbolId, &'a HirExpr>,
) {
    for statement in statements {
        match &statement.kind {
            HirStmtKind::Expr(expr) => collect_request_expr_initializers(expr, initializers),
            HirStmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                collect_request_expr_initializers(condition, initializers);
                collect_request_stmt_initializers(then_branch, initializers);
                collect_request_stmt_initializers(else_branch, initializers);
            }
            HirStmtKind::Switch { selector, arms } => {
                if let Some(selector) = selector {
                    collect_request_expr_initializers(selector, initializers);
                }
                for arm in arms {
                    if let Some(condition) = &arm.condition {
                        collect_request_expr_initializers(condition, initializers);
                    }
                    collect_request_stmt_initializers(&arm.body, initializers);
                }
            }
            HirStmtKind::For {
                from,
                to,
                step,
                body,
                ..
            } => {
                collect_request_expr_initializers(from, initializers);
                collect_request_expr_initializers(to, initializers);
                if let Some(step) = step {
                    collect_request_expr_initializers(step, initializers);
                }
                collect_request_stmt_initializers(body, initializers);
            }
            HirStmtKind::ForIn { iterable, body, .. } => {
                collect_request_expr_initializers(iterable, initializers);
                collect_request_stmt_initializers(body, initializers);
            }
            HirStmtKind::While { condition, body } => {
                collect_request_expr_initializers(condition, initializers);
                collect_request_stmt_initializers(body, initializers);
            }
            HirStmtKind::Decl { symbol, value } => {
                initializers.insert(*symbol, value);
                collect_request_expr_initializers(value, initializers);
            }
            HirStmtKind::Reassign { value, .. } | HirStmtKind::FieldReassign { value, .. } => {
                collect_request_expr_initializers(value, initializers);
            }
            HirStmtKind::ArrayFieldReassign {
                array,
                index,
                value,
                ..
            } => {
                collect_request_expr_initializers(array, initializers);
                collect_request_expr_initializers(index, initializers);
                collect_request_expr_initializers(value, initializers);
            }
            HirStmtKind::TupleDecl { value, .. } => {
                collect_request_expr_initializers(value, initializers);
            }
            HirStmtKind::Break | HirStmtKind::Continue => {}
        }
    }
}

fn collect_request_expr_initializers<'a>(
    expression: &'a HirExpr,
    initializers: &mut HashMap<SymbolId, &'a HirExpr>,
) {
    match &expression.kind {
        HirExprKind::Literal(_) | HirExprKind::Builtin(_) | HirExprKind::Symbol(_) => {}
        HirExprKind::Unary { expr, .. } => collect_request_expr_initializers(expr, initializers),
        HirExprKind::Binary { left, right, .. } => {
            collect_request_expr_initializers(left, initializers);
            collect_request_expr_initializers(right, initializers);
        }
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_request_expr_initializers(condition, initializers);
            collect_request_expr_initializers(then_expr, initializers);
            collect_request_expr_initializers(else_expr, initializers);
        }
        HirExprKind::Switch { selector, arms } => {
            if let Some(selector) = selector {
                collect_request_expr_initializers(selector, initializers);
            }
            for arm in arms {
                if let Some(condition) = &arm.condition {
                    collect_request_expr_initializers(condition, initializers);
                }
                collect_request_expr_initializers(&arm.result, initializers);
            }
        }
        HirExprKind::For {
            from,
            to,
            step,
            statements,
            result,
            ..
        } => {
            collect_request_expr_initializers(from, initializers);
            collect_request_expr_initializers(to, initializers);
            if let Some(step) = step {
                collect_request_expr_initializers(step, initializers);
            }
            collect_request_stmt_initializers(statements, initializers);
            collect_request_expr_initializers(result, initializers);
        }
        HirExprKind::ForIn {
            iterable,
            statements,
            result,
            ..
        } => {
            collect_request_expr_initializers(iterable, initializers);
            collect_request_stmt_initializers(statements, initializers);
            collect_request_expr_initializers(result, initializers);
        }
        HirExprKind::While {
            condition,
            statements,
            result,
        } => {
            collect_request_expr_initializers(condition, initializers);
            collect_request_stmt_initializers(statements, initializers);
            collect_request_expr_initializers(result, initializers);
        }
        HirExprKind::Tuple(items) => {
            for item in items {
                collect_request_expr_initializers(item, initializers);
            }
        }
        HirExprKind::UserTypeConstruct { fields, .. } => {
            for field in fields {
                collect_request_expr_initializers(field, initializers);
            }
        }
        HirExprKind::UserTypeArrayConstruct { elements, .. } => {
            for element in elements {
                collect_request_expr_initializers(element, initializers);
            }
        }
        HirExprKind::FieldAccess { value, .. } => {
            collect_request_expr_initializers(value, initializers);
        }
        HirExprKind::Block { statements, result } => {
            collect_request_stmt_initializers(statements, initializers);
            collect_request_expr_initializers(result, initializers);
        }
        HirExprKind::Call { args, .. } => {
            for arg in args {
                collect_request_expr_initializers(&arg.value, initializers);
            }
        }
        HirExprKind::History { expr, offset } => {
            collect_request_expr_initializers(expr, initializers);
            if let HirHistoryOffset::Dynamic(offset) = offset {
                collect_request_expr_initializers(offset, initializers);
            }
        }
    }
}

fn collect_hir_expr_symbols(expression: &HirExpr, symbols: &mut Vec<SymbolId>) {
    collect_hir_expr_symbols_inner(expression, symbols, &mut HashSet::new());
}

fn collect_hir_expr_symbols_inner(
    expression: &HirExpr,
    symbols: &mut Vec<SymbolId>,
    bound: &mut HashSet<SymbolId>,
) {
    match &expression.kind {
        HirExprKind::Literal(_) | HirExprKind::Builtin(_) => {}
        HirExprKind::Symbol(symbol) => {
            if !bound.contains(symbol) {
                symbols.push(*symbol);
            }
        }
        HirExprKind::Unary { expr, .. } => collect_hir_expr_symbols_inner(expr, symbols, bound),
        HirExprKind::Binary { left, right, .. } => {
            collect_hir_expr_symbols_inner(left, symbols, bound);
            collect_hir_expr_symbols_inner(right, symbols, bound);
        }
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_hir_expr_symbols_inner(condition, symbols, bound);
            collect_hir_expr_symbols_inner(then_expr, symbols, bound);
            collect_hir_expr_symbols_inner(else_expr, symbols, bound);
        }
        HirExprKind::Switch { selector, arms } => {
            if let Some(selector) = selector {
                collect_hir_expr_symbols_inner(selector, symbols, bound);
            }
            for arm in arms {
                if let Some(condition) = &arm.condition {
                    collect_hir_expr_symbols_inner(condition, symbols, bound);
                }
                collect_hir_expr_symbols_inner(&arm.result, symbols, bound);
            }
        }
        HirExprKind::For {
            counter,
            from,
            to,
            step,
            statements,
            result,
            ..
        } => {
            collect_hir_expr_symbols_inner(from, symbols, bound);
            collect_hir_expr_symbols_inner(to, symbols, bound);
            if let Some(step) = step {
                collect_hir_expr_symbols_inner(step, symbols, bound);
            }
            bound.insert(*counter);
            collect_hir_stmt_symbols(statements, symbols, bound);
            collect_hir_expr_symbols_inner(result, symbols, bound);
        }
        HirExprKind::ForIn {
            index,
            value,
            iterable,
            statements,
            result,
            ..
        } => {
            collect_hir_expr_symbols_inner(iterable, symbols, bound);
            if let Some(index) = index {
                bound.insert(*index);
            }
            bound.insert(*value);
            collect_hir_stmt_symbols(statements, symbols, bound);
            collect_hir_expr_symbols_inner(result, symbols, bound);
        }
        HirExprKind::While {
            condition,
            statements,
            result,
        } => {
            collect_hir_expr_symbols_inner(condition, symbols, bound);
            collect_hir_stmt_symbols(statements, symbols, bound);
            collect_hir_expr_symbols_inner(result, symbols, bound);
        }
        HirExprKind::Tuple(items) => {
            for item in items {
                collect_hir_expr_symbols_inner(item, symbols, bound);
            }
        }
        HirExprKind::UserTypeConstruct { fields, .. } => {
            for field in fields {
                collect_hir_expr_symbols_inner(field, symbols, bound);
            }
        }
        HirExprKind::UserTypeArrayConstruct { elements, .. } => {
            for element in elements {
                collect_hir_expr_symbols_inner(element, symbols, bound);
            }
        }
        HirExprKind::FieldAccess { value, .. } => {
            collect_hir_expr_symbols_inner(value, symbols, bound)
        }
        HirExprKind::Block { statements, result } => {
            collect_hir_stmt_symbols(statements, symbols, bound);
            collect_hir_expr_symbols_inner(result, symbols, bound);
        }
        HirExprKind::Call { args, .. } => {
            for arg in args {
                collect_hir_expr_symbols_inner(&arg.value, symbols, bound);
            }
        }
        HirExprKind::History { expr, offset } => {
            collect_hir_expr_symbols_inner(expr, symbols, bound);
            if let HirHistoryOffset::Dynamic(offset) = offset {
                collect_hir_expr_symbols_inner(offset, symbols, bound);
            }
        }
    }
}

fn collect_hir_stmt_symbols(
    statements: &[HirStmt],
    symbols: &mut Vec<SymbolId>,
    bound: &mut HashSet<SymbolId>,
) {
    for statement in statements {
        match &statement.kind {
            HirStmtKind::Expr(expr) => collect_hir_expr_symbols_inner(expr, symbols, bound),
            HirStmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                collect_hir_expr_symbols_inner(condition, symbols, bound);
                collect_hir_stmt_symbols(then_branch, symbols, bound);
                collect_hir_stmt_symbols(else_branch, symbols, bound);
            }
            HirStmtKind::Switch { selector, arms } => {
                if let Some(selector) = selector {
                    collect_hir_expr_symbols_inner(selector, symbols, bound);
                }
                for arm in arms {
                    if let Some(condition) = &arm.condition {
                        collect_hir_expr_symbols_inner(condition, symbols, bound);
                    }
                    collect_hir_stmt_symbols(&arm.body, symbols, bound);
                }
            }
            HirStmtKind::For {
                counter,
                from,
                to,
                step,
                body,
                ..
            } => {
                collect_hir_expr_symbols_inner(from, symbols, bound);
                collect_hir_expr_symbols_inner(to, symbols, bound);
                if let Some(step) = step {
                    collect_hir_expr_symbols_inner(step, symbols, bound);
                }
                bound.insert(*counter);
                collect_hir_stmt_symbols(body, symbols, bound);
            }
            HirStmtKind::ForIn {
                index,
                value,
                iterable,
                body,
            } => {
                collect_hir_expr_symbols_inner(iterable, symbols, bound);
                if let Some(index) = index {
                    bound.insert(*index);
                }
                bound.insert(*value);
                collect_hir_stmt_symbols(body, symbols, bound);
            }
            HirStmtKind::While { condition, body } => {
                collect_hir_expr_symbols_inner(condition, symbols, bound);
                collect_hir_stmt_symbols(body, symbols, bound);
            }
            HirStmtKind::Decl { symbol, value } => {
                collect_hir_expr_symbols_inner(value, symbols, bound);
                bound.insert(*symbol);
            }
            HirStmtKind::Reassign { value, .. } | HirStmtKind::FieldReassign { value, .. } => {
                collect_hir_expr_symbols_inner(value, symbols, bound);
            }
            HirStmtKind::ArrayFieldReassign {
                array,
                index,
                value,
                ..
            } => {
                collect_hir_expr_symbols_inner(array, symbols, bound);
                collect_hir_expr_symbols_inner(index, symbols, bound);
                collect_hir_expr_symbols_inner(value, symbols, bound);
            }
            HirStmtKind::TupleDecl {
                symbols: declared,
                value,
            } => {
                collect_hir_expr_symbols_inner(value, symbols, bound);
                bound.extend(declared.iter().copied());
            }
            HirStmtKind::Break | HirStmtKind::Continue => {}
        }
    }
}

fn validate_provider_timeframe(
    symbol: &str,
    requested_timeframe: &RequestTimeframe,
    chart_timeframe: &RequestTimeframe,
) -> Result<(), RuntimeError> {
    if requested_timeframe.seconds() < chart_timeframe.seconds() {
        return Err(RuntimeError {
            message: format!(
                "request.security lower timeframe requests are not supported for symbol `{symbol}` timeframe `{}` on chart timeframe `{}`",
                requested_timeframe.value(),
                chart_timeframe.value()
            ),
        });
    }
    if requested_timeframe.seconds() % chart_timeframe.seconds() != 0 {
        return Err(RuntimeError {
            message: format!(
                "request.security requested timeframe `{}` must be an integer multiple of chart timeframe `{}`",
                requested_timeframe.value(),
                chart_timeframe.value()
            ),
        });
    }
    Ok(())
}

fn align_requested_value(
    requested_values: &AppendHistory<(i64, PineValue)>,
    current_time: i64,
    requested_timeframe: &RequestTimeframe,
    chart_timeframe: &RequestTimeframe,
    merge: RequestMergePolicy,
    update_kind: BarUpdateKind,
) -> PineValue {
    let by_open = requested_timeframe == chart_timeframe
        || (merge.lookahead == RequestLookahead::On && update_kind == BarUpdateKind::Historical);
    let target = if by_open {
        current_time
    } else {
        request_bar_nominal_close(current_time, chart_timeframe)
    };
    let stamp = |index| {
        if by_open {
            requested_values[index].0
        } else {
            requested_bar_close(requested_values, index, requested_timeframe)
        }
    };
    let (mut lo, mut hi) = (0, requested_values.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let before = if merge.gaps == RequestGaps::On {
            stamp(mid) < target
        } else {
            stamp(mid) <= target
        };
        if before {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    let index = if merge.gaps == RequestGaps::On {
        if lo == requested_values.len() || stamp(lo) != target {
            return PineValue::Na;
        }
        lo
    } else {
        let Some(index) = lo.checked_sub(1) else {
            return PineValue::Na;
        };
        index
    };
    requested_values[index].1.clone()
}

fn request_bar_nominal_close(open_time: i64, timeframe: &RequestTimeframe) -> i64 {
    calendar_timeframe_close(open_time, timeframe.value(), timeframe.seconds())
        .unwrap_or_else(|| open_time.saturating_add(timeframe.seconds().saturating_mul(1000)))
}

fn requested_bar_close(
    requested_values: &AppendHistory<(i64, PineValue)>,
    index: usize,
    timeframe: &RequestTimeframe,
) -> i64 {
    let open_time = requested_values[index].0;
    let nominal_close = request_bar_nominal_close(open_time, timeframe);
    requested_values
        .get(index + 1)
        .map(|(next_open, _)| *next_open)
        .filter(|next_open| *next_open > open_time)
        .map_or(nominal_close, |next_open| nominal_close.min(next_open))
}

fn legacy_source_span(args: &[HirCallArg]) -> Option<(i64, i64)> {
    let literal = |name: &str| {
        args.iter()
            .find(|arg| arg.name.as_deref() == Some(name))
            .and_then(|arg| match arg.value.kind {
                pine_ir::HirExprKind::Literal(pine_ir::HirLiteral::Int(value)) => Some(value),
                _ => None,
            })
    };
    Some((literal("$legacy_span_start")?, literal("$legacy_span_end")?))
}
