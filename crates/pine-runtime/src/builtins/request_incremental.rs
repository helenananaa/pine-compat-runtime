//! Reuse a checkpoint preceding the terminal requested bar. The old terminal
//! bar is always replayed when a context grows, preserving endpoint semantics.
use super::request_values::RequestedValue;
use super::requests::request_capture_values;
use crate::request::RequestCacheKey;
use crate::runtime::append_history::AppendHistory;
use crate::runtime::request_dependencies::RequestDependencies;
use crate::{
    HistoricalRuntime, PineValue, RequestDataError, RequestEnvironment, RequestKey, RuntimeError,
};
use pine_ir::{HirExpr, HirExprKind, HirHistoryOffset, HirStmt, HirStmtKind, SymbolId};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone)]
pub(crate) struct RequestEvaluation<'a> {
    prefix: AppendHistory<(i64, RequestedValue)>,
    before_last: Arc<HistoricalRuntime<'a>>,
    captures: HashMap<SymbolId, PineValue>,
    provider_len: usize,
    window_start: usize,
    #[cfg(test)]
    replayed_bars: usize,
}

impl RequestEvaluation<'_> {
    pub(crate) fn cached_values_profile(&self) -> (usize, usize) {
        (self.prefix.len(), self.prefix.capacity())
    }
    pub(crate) fn capture_values(&self) -> impl Iterator<Item = &PineValue> {
        self.captures.values()
    }

    pub(crate) fn valuewhen_retained_values(&self) -> usize {
        self.before_last.valuewhen_retained_values()
    }

    #[cfg(test)]
    pub(crate) fn valuewhen_checkpoint(&self) -> &HistoricalRuntime<'_> {
        &self.before_last
    }
}

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn evaluate_request_incremental(
        &mut self,
        key: &RequestKey,
        cache_key: &RequestCacheKey,
        expression: &HirExpr,
        environment: RequestEnvironment,
        include_forming: bool,
    ) -> Result<AppendHistory<(i64, RequestedValue)>, RuntimeError> {
        let program = self.program.clone();
        let metadata = Arc::clone(&self.metadata);
        let dependencies = metadata.request_dependencies(&program);
        let captures = request_capture_values(expression, &self.current_symbols, dependencies);
        for value in captures.values() {
            self.reject_request_object_graph(value)?;
        }
        if !self.incremental_request_admitted(cache_key, expression, dependencies, &captures) {
            let bars = self.resolved_request_bars_from(key, 0)?;
            let start = if self.context_windows.prepared {
                self.context_windows.source_start(key, &bars)
            } else {
                0
            };
            return self
                .evaluate_requested_values(&bars[start..], expression, environment)
                .map(AppendHistory::from_values);
        }
        let canonical = self
            .program
            .strategy_settings
            .calc_on_every_history_tick
            .then(|| self.static_requests.canonical_chart_bars(key))
            .flatten()
            .cloned();
        let provider_bars = if canonical.is_some() {
            &[]
        } else {
            match environment.provider().bars(key) {
                Ok(bars) => bars,
                Err(RequestDataError::MissingData { .. }) => &[],
                Err(error) => {
                    return Err(RuntimeError {
                        message: error.to_string(),
                    });
                }
            }
        };
        let provider_len = canonical
            .as_ref()
            .map_or(provider_bars.len(), AppendHistory::len);
        let total = if canonical.is_some() {
            provider_len
        } else {
            self.request_feed
                .resolved_len(key, provider_len, include_forming)
        };
        let window_start = if self.context_windows.prepared {
            match self.context_windows.anchored_origin(key) {
                Some(origin) => origin.map_or(0, |(time, offset)| {
                    canonical
                        .as_ref()
                        .map_or_else(
                            || {
                                self.request_feed.resolved_before(
                                    key,
                                    provider_bars,
                                    include_forming,
                                    time,
                                )
                            },
                            |bars| bars.partition_point(|bar| bar.time < time),
                        )
                        .saturating_add(offset)
                        .min(total)
                }),
                None => {
                    let bars = self.resolved_request_bars_from(key, 0)?;
                    self.context_windows.source_start(key, &bars)
                }
            }
        } else {
            0
        };
        let current_time = self
            .current_bar
            .ok_or_else(|| RuntimeError {
                message: "request has no current chart bar".to_owned(),
            })?
            .time;
        let horizon = self.request_environment.chart_close(current_time)?;
        // Endpoint-independent expressions only need the chart's visible
        // prefix. Retain one successor to establish shortened/irregular closes
        // during alignment; its value cannot cross the merge time boundary.
        let count = canonical
            .as_ref()
            .map_or_else(
                || {
                    self.request_feed
                        .resolved_before(key, provider_bars, include_forming, horizon)
                },
                |bars| bars.partition_point(|bar| bar.time < horizon),
            )
            .saturating_add(1)
            .min(total)
            .saturating_sub(window_start);
        if count == 0 {
            if total == 0 {
                self.resolved_request_bars_range(key, 0, 0)?;
            }
            return Ok(AppendHistory::default());
        }
        let previous = self
            .request_evaluations
            .get(cache_key)
            .filter(|saved| {
                saved.provider_len == provider_len
                    && saved.window_start == window_start
                    && saved.captures == captures
                    && saved.prefix.len() < count
            })
            .cloned();
        let replaced_values = self
            .request_evaluations
            .get(cache_key)
            .map_or(0, |saved| saved.valuewhen_retained_values());
        let mut prefix = previous
            .as_ref()
            .map_or_else(AppendHistory::default, |saved| saved.prefix.clone());
        let mut runtime = match previous {
            Some(saved) => (*saved.before_last).clone(),
            None => self.fork_with_request_environment(environment),
        };
        // Checkpoints retain Pine state, not an allowance from an older chart
        // execution. All newly evaluated requested bars spend today's budget.
        runtime.inherit_execution_budget(self);
        runtime.inherit_valuewhen_budget(self, replaced_values)?;
        let bars = self.resolved_request_bars_range(
            key,
            window_start + prefix.len(),
            window_start + count,
        )?;
        if bars.is_empty() {
            self.request_evaluations.remove(cache_key);
            return Ok(prefix);
        }
        runtime.historical_end = Some(count);
        let mut before_last = None;
        let mut last = None;
        for (index, bar) in bars.iter().enumerate() {
            let terminal = index + 1 == bars.len();
            if terminal {
                before_last = Some(Arc::new(runtime.clone()));
            }
            let result =
                runtime.eval_requested_bar_expression(*bar, expression, &captures, dependencies);
            self.accept_execution_budget(&runtime);
            if result.is_err() {
                self.failed_request_context = runtime.failed_request_context.clone();
            }
            let value = result?;
            if terminal {
                last = Some((bar.time, value));
            } else {
                prefix.push((bar.time, value));
            }
        }
        let mut values = prefix.clone();
        values.push(last.expect("resolved bars are nonempty"));
        let before_last = before_last.expect("terminal checkpoint");
        self.replace_requested_valuewhen_values(
            replaced_values,
            before_last.valuewhen_retained_values(),
        )?;
        self.request_evaluations.insert(
            cache_key.clone(),
            Arc::new(RequestEvaluation {
                prefix,
                before_last,
                captures,
                provider_len,
                window_start,
                #[cfg(test)]
                replayed_bars: bars.len(),
            }),
        );
        Ok(values)
    }

    /// Admission reads which dependencies are captured, not their values, so
    /// it is checked once per request and captured symbol set.
    fn incremental_request_admitted(
        &mut self,
        cache_key: &RequestCacheKey,
        expression: &HirExpr,
        dependencies: RequestDependencies<'_>,
        captures: &HashMap<SymbolId, PineValue>,
    ) -> bool {
        #[cfg(test)]
        if COMPLETE_EVALUATOR.with(std::cell::Cell::get) {
            return false;
        }
        let mut captured: Box<[SymbolId]> = captures.keys().copied().collect();
        captured.sort_unstable_by_key(|symbol| symbol.0);
        if let Some((checked, admitted)) = self.request_incremental_admissions.get(cache_key)
            && *checked == captured
        {
            return *admitted;
        }
        let admitted = IncrementalCheck {
            dependencies,
            captures,
            checked: HashMap::new(),
            loops: 0,
        }
        .expression(expression, &HashSet::new(), 0);
        self.request_incremental_admissions
            .insert(cache_key.clone(), (captured, admitted));
        admitted
    }
}

#[cfg(test)]
thread_local! {
    static INITIALIZER_CHECKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    // Differential tests run one of two runtimes with every request, nested
    // ones included, on the complete evaluator.
    static COMPLETE_EVALUATOR: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

// Replay only expressions whose effects live in the requested checkpoint and
// whose inputs do not depend on the dataset endpoint. Other expressions retain
// the complete evaluator.
struct IncrementalCheck<'p, 'c> {
    dependencies: RequestDependencies<'p>,
    captures: &'c HashMap<SymbolId, PineValue>,
    // Dependency graphs share initializers across paths. Outside a block, a
    // symbol's answer depends only on the depth that reaches it.
    checked: HashMap<(SymbolId, usize), bool>,
    // Admitted loop bodies enclosing the statement; `break` and `continue`
    // need one.
    loops: usize,
}

impl IncrementalCheck<'_, '_> {
    fn expression(&mut self, expr: &HirExpr, locals: &HashSet<SymbolId>, depth: usize) -> bool {
        if depth > 64 {
            return false;
        }
        let next = depth + 1;
        match &expr.kind {
            HirExprKind::Literal(_) => true,
            HirExprKind::Builtin(name) => stable_builtin(name),
            HirExprKind::Symbol(symbol) => self.symbol(*symbol, locals, depth),
            HirExprKind::Unary { expr, .. } => self.expression(expr, locals, next),
            HirExprKind::Binary { left, right, .. } => {
                self.expression(left, locals, next) && self.expression(right, locals, next)
            }
            HirExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.expression(condition, locals, next)
                    && self.expression(then_expr, locals, next)
                    && self.expression(else_expr, locals, next)
            }
            HirExprKind::Tuple(items) => {
                items.iter().all(|item| self.expression(item, locals, next))
            }
            HirExprKind::History { expr, offset } => {
                self.expression(expr, locals, next)
                    && match offset {
                        HirHistoryOffset::Constant(_) => true,
                        HirHistoryOffset::Dynamic(value) => self.expression(value, locals, next),
                    }
            }
            HirExprKind::Call { callee, args, .. } => {
                (callee == "input"
                    || callee.starts_with("input.")
                    || callee.starts_with("ta.")
                    || (callee.starts_with("math.") && callee != "math.random")
                    || pine_builtins::is_pure_scalar_string_builtin(callee)
                    || matches!(
                        callee.as_str(),
                        "timeframe.change"
                            | "year"
                            | "nz"
                            | "na"
                            | "fixnan"
                            | "int"
                            | "float"
                            | "bool"
                            | "color.new"
                            | "color.rgb"
                            | "color.r"
                            | "color.g"
                            | "color.b"
                            | "color.t"
                    ))
                    && args
                        .iter()
                        .all(|arg| self.expression(&arg.value, locals, next))
            }
            HirExprKind::Block { statements, result } => {
                let mut bound = locals.clone();
                statements
                    .iter()
                    .all(|statement| self.statement(statement, &mut bound, next))
                    && self.expression(result, &bound, next)
            }
            HirExprKind::For {
                counter,
                from,
                to,
                step,
                statements,
                result,
            } => {
                self.expression(from, locals, next)
                    && self.expression(to, locals, next)
                    && step
                        .as_deref()
                        .is_none_or(|step| self.expression(step, locals, next))
                    && self.loop_body(Some(*counter), statements, Some(result), locals, next)
            }
            HirExprKind::While {
                condition,
                statements,
                result,
            } => {
                self.expression(condition, locals, next)
                    && self.loop_body(None, statements, Some(result), locals, next)
            }
            _ => false,
        }
    }

    // Counters, body locals and loop-local `var` state live in the checkpoint
    // like other block locals. Bounds and conditions read the enclosing scope.
    fn loop_body(
        &mut self,
        counter: Option<SymbolId>,
        statements: &[HirStmt],
        result: Option<&HirExpr>,
        locals: &HashSet<SymbolId>,
        depth: usize,
    ) -> bool {
        let mut bound = locals.clone();
        bound.extend(counter);
        self.loops += 1;
        let admitted = statements
            .iter()
            .all(|statement| self.statement(statement, &mut bound, depth))
            && result.is_none_or(|result| self.expression(result, &bound, depth));
        self.loops -= 1;
        admitted
    }

    fn symbol(&mut self, symbol: SymbolId, locals: &HashSet<SymbolId>, depth: usize) -> bool {
        if locals.contains(&symbol) || self.captures.contains_key(&symbol) {
            return true;
        }
        let Some(initializer) = self.dependencies.initializer(symbol) else {
            return self
                .dependencies
                .symbol(symbol)
                .is_some_and(|item| stable_builtin(&item.name));
        };
        // Block locals reach initializers unchanged; keep those paths exact.
        if !locals.is_empty() {
            return self.initializer(initializer, locals, depth);
        }
        if let Some(&admitted) = self.checked.get(&(symbol, depth)) {
            return admitted;
        }
        let admitted = self.initializer(initializer, locals, depth);
        self.checked.insert((symbol, depth), admitted);
        admitted
    }

    fn initializer(&mut self, value: &HirExpr, locals: &HashSet<SymbolId>, depth: usize) -> bool {
        #[cfg(test)]
        INITIALIZER_CHECKS.with(|checks| checks.set(checks.get() + 1));
        self.expression(value, locals, depth + 1)
    }

    // Lowered scalar UDFs keep their local state in the checkpoint. Admit only
    // declarations, local assignments, conditional branches and counted or
    // conditional loops with stable inputs. Mutations of external state,
    // for...in loops, nested requests and endpoint reads retain the complete
    // evaluator.
    fn statement(
        &mut self,
        statement: &HirStmt,
        locals: &mut HashSet<SymbolId>,
        depth: usize,
    ) -> bool {
        if depth > 64 {
            return false;
        }
        let next = depth + 1;
        match &statement.kind {
            HirStmtKind::Decl { symbol, value } => {
                if !self.expression(value, locals, next) {
                    return false;
                }
                locals.insert(*symbol);
                true
            }
            HirStmtKind::Reassign { symbol, value } => {
                locals.contains(symbol) && self.expression(value, locals, next)
            }
            HirStmtKind::Expr(value) => self.expression(value, locals, next),
            HirStmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.expression(condition, locals, next)
                    && [then_branch, else_branch].into_iter().all(|branch| {
                        let mut bound = locals.clone();
                        branch
                            .iter()
                            .all(|statement| self.statement(statement, &mut bound, next))
                    })
            }
            HirStmtKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => {
                self.expression(from, locals, next)
                    && self.expression(to, locals, next)
                    && step
                        .as_ref()
                        .is_none_or(|step| self.expression(step, locals, next))
                    && self.loop_body(Some(*counter), body, None, locals, next)
            }
            HirStmtKind::While { condition, body } => {
                self.expression(condition, locals, next)
                    && self.loop_body(None, body, None, locals, next)
            }
            HirStmtKind::Break | HirStmtKind::Continue => self.loops > 0,
            _ => false,
        }
    }
}

fn stable_builtin(name: &str) -> bool {
    matches!(
        name,
        "na" | "open"
            | "high"
            | "low"
            | "close"
            | "volume"
            | "time"
            | "time_tradingday"
            | "bar_index"
            | "hl2"
            | "hlc3"
            | "ohlc4"
            | "hlcc4"
            | "barstate.isfirst"
    ) || name.starts_with("syminfo.")
        || name.starts_with("timeframe.")
}

#[cfg(test)]
#[path = "request_incremental_tests.rs"]
mod differential_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Bar, BarUpdate, BarUpdateKind, ChartContext, InMemoryRequestDataProvider, RequestTimeframe,
    };
    use pine_sema::analyze_source;
    use pine_syntax::SourceFile;

    fn bar(time: i64, close: f64) -> Bar {
        Bar {
            time,
            open: close,
            high: close,
            low: close,
            close,
            volume: 1.0,
        }
    }

    #[test]
    fn realtime_static_root_refresh_retains_direct_requested_checkpoints() {
        use crate::{
            ExternalAccountFrame, ExternalBarState, ExternalEvaluationTrigger,
            ExternalExecutionOptions, ExternalPass, ExternalPassContext, ExternalScheduledPass,
        };
        let analysis = analyze_source(&SourceFile::new(
            "static-root-checkpoint",
            r#"//@version=6
strategy("Static root checkpoint",calc_on_every_tick=true,dynamic_requests=false)
plot(request.security("B","1",ta.sma(close,2)))
x=request.security(syminfo.tickerid,timeframe.period,close)
plot(request.security("B","1",x))
"#,
        ));
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.unwrap();
        let tf = RequestTimeframe::parse("1").unwrap();
        let provider = InMemoryRequestDataProvider::from_streams([(
            crate::RequestKey::new("B", tf.clone()),
            (0..64)
                .map(|index| bar(index * 60_000, (index + 1) as f64))
                .collect(),
        )])
        .unwrap();
        let environment = RequestEnvironment::new(ChartContext::new("A", tf), Arc::new(provider));
        let time = 20 * 60_000;
        let account = ExternalAccountFrame {
            time,
            position_size: 0.0,
            position_avg_price: None,
            equity: 1000.0,
            initial_capital: 1000.0,
            netprofit: 0.0,
            openprofit: 0.0,
        };
        let group = [10.0, 11.0, 12.0]
            .into_iter()
            .enumerate()
            .map(|(index, close)| ExternalScheduledPass {
                evaluation: ExternalPass {
                    bar: Bar {
                        time,
                        open: 10.0,
                        high: close,
                        low: 10.0,
                        close,
                        volume: 100.0,
                    },
                    account: account.clone(),
                    event_time_ms: time + 59999 * index as i64 / 2,
                    confirmed: index == 2,
                    request_data: None,
                    request_updates: None,
                },
                context: ExternalPassContext {
                    bar_state: ExternalBarState::Realtime,
                    trigger: if index == 2 {
                        ExternalEvaluationTrigger::BarClose
                    } else {
                        ExternalEvaluationTrigger::RealtimeTick
                    },
                    is_new: index == 0,
                },
            })
            .collect();
        let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment)
            .with_external_schedule(
                vec![account],
                vec![group],
                ExternalExecutionOptions::default(),
            )
            .unwrap();
        runtime
            .append_bar(Bar {
                time,
                open: 10.0,
                high: 12.0,
                low: 10.0,
                close: 12.0,
                volume: 100.0,
            })
            .unwrap();
        assert_eq!(runtime.request_evaluations.len(), 1);
        let cached = runtime.request_evaluations.values().next().unwrap();
        assert_eq!(
            cached.replayed_bars, 1,
            "refresh must not rebuild the 22-bar direct requested prefix"
        );
        assert_eq!(runtime.result().plots[1].values[0].as_f64(), Some(12.0));
    }

    #[test]
    fn shared_dependency_graph_is_admitted_once_per_capture_set() {
        const DEPTH: usize = 16;
        // `x{i} = x{i-1} + x{i-1}` doubles the paths to `x0` at every level.
        let mut source = String::from(
            "//@version=6
indicator(\"shared\")
x0 = close
",
        );
        for level in 1..=DEPTH {
            let previous = level - 1;
            source.push_str(&format!(
                "x{level} = x{previous} + x{previous}
"
            ));
        }
        source.push_str(&format!(
            "plot(request.security(\"B\", \"1\", x{DEPTH}))
"
        ));
        let analysis = analyze_source(&SourceFile::new("shared.pine", source));
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.unwrap();
        let timeframe = RequestTimeframe::parse("1").unwrap();
        let provider = InMemoryRequestDataProvider::from_streams([(
            RequestKey::new("B", timeframe.clone()),
            (0..8).map(|i| bar(i * 60_000, (i + 1) as f64)).collect(),
        )])
        .unwrap();
        let environment =
            RequestEnvironment::new(ChartContext::new("A", timeframe), Arc::new(provider));
        let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment);
        INITIALIZER_CHECKS.with(|checks| checks.set(0));
        for index in 0..8 {
            runtime.append_bar(bar(index * 60_000, 1.0)).unwrap();
        }
        let checks = INITIALIZER_CHECKS.with(std::cell::Cell::get);
        assert_eq!(runtime.request_evaluations.len(), 1);
        for (index, value) in runtime.result().plots[0].values.iter().enumerate() {
            assert_eq!(
                value.as_f64(),
                Some((index + 1) as f64 * (1u64 << DEPTH) as f64)
            );
        }
        // The captured set never changes, so one admission check visits each
        // initializer once. The per-path walk made 2^(DEPTH + 1) - 1 = 131,071
        // initializer checks on every bar.
        assert!(
            checks <= DEPTH + 1,
            "{checks} initializer checks for depth {DEPTH}"
        );
    }

    #[test]
    fn external_observed_growth_replays_only_the_terminal_requested_bar_and_new_tail() {
        let analysis = analyze_source(&SourceFile::new(
            "observed-growth",
            "//@version=6\nstrategy(\"growth\")\nplot(request.security(\"B\",\"1\",ta.cum(close)))",
        ));
        assert!(analysis.diagnostics.is_empty());
        let hir = analysis.hir.unwrap();
        let timeframe = RequestTimeframe::parse("1").unwrap();
        let provider = InMemoryRequestDataProvider::from_streams([(
            RequestKey::new("B", timeframe.clone()),
            vec![],
        )])
        .unwrap();
        let environment =
            RequestEnvironment::new(ChartContext::new("A", timeframe), Arc::new(provider));
        let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment)
            .with_external_accounts(vec![])
            .unwrap();
        let mut replayed = 0;
        for index in 0..512 {
            let observed = (index + 1) * 60_000 - 1;
            runtime
                .apply_external_request_updates(
                    &[crate::ObservedRequestUpdate {
                        symbol: "B".into(),
                        timeframe: "1".into(),
                        bar: bar(index * 60_000, (index + 1) as f64),
                        kind: BarUpdateKind::Confirmed,
                        available_at_ms: observed,
                    }],
                    observed,
                )
                .unwrap();
            runtime
                .append_external_bars(
                    &[bar(index * 60_000, 1.0)],
                    vec![crate::ExternalAccountFrame {
                        time: index * 60_000,
                        position_size: 0.0,
                        position_avg_price: None,
                        equity: 1000.0,
                        initial_capital: 1000.0,
                        netprofit: 0.0,
                        openprofit: 0.0,
                    }],
                    None,
                )
                .unwrap();
            assert_eq!(runtime.request_evaluations.len(), 1);
            let saved = runtime.request_evaluations.values().next().unwrap();
            assert!(saved.replayed_bars <= 2);
            replayed += saved.replayed_bars;
            assert_eq!(
                runtime.result().plots[0]
                    .values
                    .last()
                    .and_then(PineValue::as_f64),
                Some(((index + 1) * (index + 2) / 2) as f64)
            );
        }
        assert_eq!(replayed, 1023);
        assert!(runtime.request_cache.is_empty());
    }

    #[test]
    fn historical_prefix_matches_complete_provider_replay_with_irregular_closes() {
        for timeframe in ["1", "11"] {
            for gaps in ["gaps_on", "gaps_off"] {
                for lookahead in ["lookahead_on", "lookahead_off"] {
                    let script = |expression: &str| {
                        format!(
                            "//@version=6\nindicator(\"prefix\")\nplot(request.security(\"B\",\"{timeframe}\",{expression},gaps=barmerge.{gaps},lookahead=barmerge.{lookahead}))\n"
                        )
                    };
                    let fast_hir =
                        analyze_source(&SourceFile::new("prefix.pine", script("ta.cum(close)")))
                            .hir
                            .unwrap();
                    let full_hir = analyze_source(&SourceFile::new(
                        "complete.pine",
                        script("ta.cum(close)+(barstate.islast?0.0:0.0)"),
                    ))
                    .hir
                    .unwrap();
                    let key = RequestKey::new("B", RequestTimeframe::parse(timeframe).unwrap());
                    let mut time = 0;
                    let provider = InMemoryRequestDataProvider::from_streams(vec![(
                        key,
                        (0..120)
                            .map(|i| {
                                let result = bar(time, i as f64 + 1.0);
                                time += [120_000, 180_000, 900_000][i % 3];
                                result
                            })
                            .collect(),
                    )])
                    .unwrap();
                    let environment = RequestEnvironment::new(
                        ChartContext::new("A", RequestTimeframe::parse("10").unwrap()),
                        Arc::new(provider),
                    );
                    let mut fast =
                        HistoricalRuntime::with_request_environment(&fast_hir, environment.clone());
                    let mut full =
                        HistoricalRuntime::with_request_environment(&full_hir, environment);
                    for index in 0..60 {
                        fast.append_bar(bar(index * 600_000, 1.0)).unwrap();
                        full.append_bar(bar(index * 600_000, 1.0)).unwrap();
                        assert_eq!(
                            fast.result().plots,
                            full.result().plots,
                            "{timeframe}/{gaps}/{lookahead}/{index}"
                        );
                    }
                    assert!(fast.request_cache.is_empty());
                    assert!(!fast.request_evaluations.is_empty());
                    assert!(full.request_evaluations.is_empty());
                }
            }
        }
    }

    #[test]
    fn twelve_long_providers_bootstrap_only_visible_prefix_under_default_budget() {
        let source = format!(
            "//@version=6\nindicator(\"budget\")\n{}",
            (0..12)
                .map(|i| format!("plot(request.security(\"B{i}\",\"1\",ta.cum(close)))\n"))
                .collect::<String>()
        );
        let hir = analyze_source(&SourceFile::new("budget.pine", source))
            .hir
            .unwrap();
        let provider = InMemoryRequestDataProvider::from_streams(
            (0..12)
                .map(|i| {
                    (
                        RequestKey::new(format!("B{i}"), RequestTimeframe::parse("1").unwrap()),
                        (0..5120).map(|n| bar(n * 60_000, n as f64 + 1.0)).collect(),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let environment = RequestEnvironment::new(
            ChartContext::new("A", RequestTimeframe::parse("1").unwrap()),
            Arc::new(provider),
        );
        let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment);
        for index in 0..5120 {
            runtime.append_bar(bar(index * 60_000, 1.0)).unwrap();
            assert_eq!(runtime.request_evaluations.len(), 12);
            assert!(
                runtime
                    .request_evaluations
                    .values()
                    .all(|saved| saved.replayed_bars <= 2)
            );
        }
        for plot in runtime.result().plots {
            for (index, value) in plot.values.iter().enumerate() {
                assert_eq!(
                    *value,
                    PineValue::Float(((index + 1) * (index + 2) / 2) as f64)
                );
            }
        }
    }

    #[test]
    fn live_ta_checkpoint_matches_forced_full_evaluation_with_bounded_replay() {
        for expression in [
            "ta.sma(close, 3)",
            "ta.ema(close, 5)",
            "ta.rsi(close, 7)",
            "ta.cum(close)",
            "nz(close[1], close)",
            "ta.sma(close, input.int(3))",
            "f()",
            "fixnan(close > 135 ? na : close)",
            "float(int(close))",
            "color.r(color.new(color.rgb(int(close), 20, 30), 25))",
            "str.tonumber(str.tostring(close))",
            "str.length(str.upper(str.format(\"value={0}\", close)))",
            r#"str.length(str.match(str.tostring(close), close % 2 == 0 ? "[0-9]+" : "[0-9]+\\.[0-9]+"))"#,
            "str.tonumber(str.replace_all(str.tostring(close), \".\", \"\"))",
        ] {
            let source = format!(
                "//@version=6\nindicator(\"cache\")\nf()=>\n    value=0.0\n    value:=nz(value[1])+nz(close[1],close)\n    value\nplot(request.security(\"B\", \"5\", {expression}))"
            );
            let analysis = analyze_source(&SourceFile::new("cache.pine", source));
            assert!(
                analysis.diagnostics.is_empty(),
                "{expression}: {:?}",
                analysis.diagnostics
            );
            let hir = analysis.hir.expect("supported request expression");
            let key = RequestKey::new("B", RequestTimeframe::parse("5").unwrap());
            let mut provider = InMemoryRequestDataProvider::new();
            provider
                .insert(
                    key.clone(),
                    (0..128)
                        .map(|i| bar(i * 300_000, 10.0 + i as f64))
                        .collect(),
                )
                .unwrap();
            let environment = RequestEnvironment::new(
                ChartContext::new("A", RequestTimeframe::parse("1").unwrap()),
                Arc::new(provider),
            );
            let mut fast = HistoricalRuntime::with_request_environment(&hir, environment.clone());
            let mut full = HistoricalRuntime::with_request_environment(&hir, environment);
            fast.append_bar(bar(127 * 300_000 + 240_000, 1.0)).unwrap();
            full.append_bar(bar(127 * 300_000 + 240_000, 1.0)).unwrap();
            for index in 128..144 {
                for bump in [0.0, 0.5, 1.0] {
                    let requested = bar(index * 300_000, index as f64 + bump);
                    fast.apply_request_update(key.clone(), BarUpdate::forming(requested))
                        .unwrap();
                    full.apply_request_update(key.clone(), BarUpdate::forming(requested))
                        .unwrap();
                    full.clear_request_evaluations();
                    let chart = bar(index * 300_000 + 240_000, 1.0);
                    fast.append_bar_with_kind(chart, BarUpdateKind::Forming)
                        .unwrap();
                    full.append_bar_with_kind(chart, BarUpdateKind::Forming)
                        .unwrap();
                    assert_eq!(
                        fast.result().plots,
                        full.result().plots,
                        "{expression}/{index}/{bump}"
                    );
                    assert!(
                        !fast.request_evaluations.is_empty(),
                        "{:#?}",
                        hir.statements
                    );
                    assert!(
                        fast.request_evaluations
                            .values()
                            .all(|state| state.replayed_bars <= 2)
                    );
                }
                let requested = bar(index * 300_000, index as f64 + 1.0);
                fast.apply_request_update(key.clone(), BarUpdate::confirmed(requested))
                    .unwrap();
                full.apply_request_update(key.clone(), BarUpdate::confirmed(requested))
                    .unwrap();
                full.clear_request_evaluations();
                let chart = bar(index * 300_000 + 240_000, 1.0);
                fast.append_bar_with_kind(chart, BarUpdateKind::Confirmed)
                    .unwrap();
                full.append_bar_with_kind(chart, BarUpdateKind::Confirmed)
                    .unwrap();
                assert_eq!(
                    fast.result().plots,
                    full.result().plots,
                    "{expression}/{index}/confirmed"
                );
                assert!(
                    fast.request_evaluations
                        .values()
                        .all(|state| state.replayed_bars <= 2)
                );
            }
        }
    }

    #[test]
    fn dataset_endpoint_expression_uses_complete_evaluator() {
        let hir = analyze_source(&SourceFile::new("endpoint.pine",
            "//@version=6\nindicator(\"end\")\nplot(request.security(\"B\", \"5\", barstate.islast ? close : close[1]))")).hir.unwrap();
        let key = RequestKey::new("B", RequestTimeframe::parse("5").unwrap());
        let mut provider = InMemoryRequestDataProvider::new();
        provider
            .insert(key, vec![bar(0, 1.), bar(300_000, 2.)])
            .unwrap();
        let environment = RequestEnvironment::new(
            ChartContext::new("A", RequestTimeframe::parse("1").unwrap()),
            Arc::new(provider),
        );
        let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment);
        runtime.append_bar(bar(240_000, 1.)).unwrap();
        assert!(runtime.request_evaluations.is_empty());
    }

    #[test]
    fn live_udf_tuple_checkpoint_matches_full_replay_with_local_counter() {
        let source = SourceFile::new(
            "udf.pine",
            "//@version=6\nindicator(\"udf\")\ncount(bool anchor) =>\n    var int n = 0\n    if anchor\n        n += 1\n    n\nvalues() =>\n    bool anchor = timeframe.change(\"5\") and year(time_tradingday, syminfo.timezone) % 1 == 0\n    [ta.pivot_point_levels(\"Traditional\", anchor), ta.pivot_point_levels(\"Traditional\", anchor, developing=true), count(anchor)]\n[a,b,n] = request.security(\"B\", \"5\", values(), lookahead=barmerge.lookahead_on)\nplot(array.get(a, 0))\nplot(array.get(b, 0))\nplot(n)\n",
        );
        let hir = analyze_source(&source).hir.unwrap();
        let key = RequestKey::new("B", RequestTimeframe::parse("5").unwrap());
        let provider = InMemoryRequestDataProvider::from_streams(vec![(
            key.clone(),
            (0..128)
                .map(|i| bar(i * 300_000, 10.0 + i as f64))
                .collect(),
        )])
        .unwrap();
        let environment = RequestEnvironment::new(
            ChartContext::new("A", RequestTimeframe::parse("1").unwrap()),
            Arc::new(provider),
        );
        let mut fast = HistoricalRuntime::with_request_environment(&hir, environment.clone());
        let mut full = HistoricalRuntime::with_request_environment(&hir, environment);
        fast.append_bar(bar(127 * 300_000 + 240_000, 1.0)).unwrap();
        full.append_bar(bar(127 * 300_000 + 240_000, 1.0)).unwrap();
        for index in 128..144 {
            for bump in [0.0, 0.5, 1.0] {
                let requested = bar(index * 300_000, index as f64 + bump);
                fast.apply_request_update(key.clone(), BarUpdate::forming(requested))
                    .unwrap();
                full.apply_request_update(key.clone(), BarUpdate::forming(requested))
                    .unwrap();
                full.clear_request_evaluations();
                let chart = bar(index * 300_000 + 240_000, 1.0);
                fast.append_bar_with_kind(chart, BarUpdateKind::Forming)
                    .unwrap();
                full.append_bar_with_kind(chart, BarUpdateKind::Forming)
                    .unwrap();
                assert_eq!(fast.result().plots, full.result().plots, "{index}/{bump}");
                assert!(!fast.request_evaluations.is_empty());
                assert!(
                    fast.request_evaluations
                        .values()
                        .all(|state| state.replayed_bars <= 2)
                );
            }
            let requested = bar(index * 300_000, index as f64 + 1.0);
            fast.apply_request_update(key.clone(), BarUpdate::confirmed(requested))
                .unwrap();
            full.apply_request_update(key.clone(), BarUpdate::confirmed(requested))
                .unwrap();
        }
    }

    #[test]
    fn endpoint_read_inside_udf_branch_keeps_complete_evaluator() {
        let source = SourceFile::new(
            "endpoint-udf.pine",
            "//@version=6\nindicator(\"end\")\nf() =>\n    var float n = 0\n    if barstate.islast\n        n += close\n    n\nplot(request.security(\"B\", \"5\", f(), lookahead=barmerge.lookahead_on))\n",
        );
        let hir = analyze_source(&source).hir.unwrap();
        let key = RequestKey::new("B", RequestTimeframe::parse("5").unwrap());
        let provider = InMemoryRequestDataProvider::from_streams(vec![(
            key,
            vec![bar(0, 1.0), bar(300_000, 2.0)],
        )])
        .unwrap();
        let environment = RequestEnvironment::new(
            ChartContext::new("A", RequestTimeframe::parse("1").unwrap()),
            Arc::new(provider),
        );
        let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment);
        runtime.append_bar(bar(240_000, 1.0)).unwrap();
        assert!(runtime.request_evaluations.is_empty());
    }

    #[test]
    fn string_endpoint_reads_inside_arguments_and_udf_branches_keep_complete_evaluator() {
        for (expression, expected_values) in [
            (
                "str.tonumber(str.tostring(barstate.islast ? close : close[1]))",
                [4.0, 6.0, 5.0],
            ),
            (
                "str.length(str.match(str.tostring(close), barstate.islast ? \"[0-9]+\" : \".\"))",
                [1.0, 1.0, 1.0],
            ),
            ("f()", [2.0, 2.0, 2.0]),
        ] {
            let source = SourceFile::new(
                "string-endpoint.pine",
                format!(
                    "//@version=6\nindicator(\"string endpoint\")\nf() =>\n    string result = str.tostring(close)\n    if barstate.islast\n        result := str.tostring(close[1])\n    str.tonumber(result)\nplot(request.security(\"B\", \"5\", {expression}, lookahead=barmerge.lookahead_on))\n"
                ),
            );
            let analysis = analyze_source(&source);
            assert!(
                analysis.diagnostics.is_empty(),
                "{expression}: {:?}",
                analysis.diagnostics
            );
            let hir = analysis.hir.unwrap();
            let key = RequestKey::new("B", RequestTimeframe::parse("5").unwrap());
            let provider = InMemoryRequestDataProvider::from_streams(vec![(
                key.clone(),
                vec![bar(0, 1.0), bar(300_000, 2.0)],
            )])
            .unwrap();
            let environment = RequestEnvironment::new(
                ChartContext::new("A", RequestTimeframe::parse("1").unwrap()),
                Arc::new(provider),
            );
            let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment);
            runtime.append_bar(bar(240_000, 1.0)).unwrap();
            for (close, expected) in [4.0, 6.0, 5.0].into_iter().zip(expected_values) {
                runtime
                    .apply_request_update(key.clone(), BarUpdate::forming(bar(600_000, close)))
                    .unwrap();
                runtime
                    .append_bar_with_kind(bar(840_000, 1.0), BarUpdateKind::Forming)
                    .unwrap();
                assert!(runtime.request_evaluations.is_empty(), "{expression}");
                assert_eq!(
                    runtime.result().plots[0]
                        .values
                        .last()
                        .and_then(PineValue::as_f64),
                    Some(expected),
                    "{expression}/{close}"
                );
            }
            runtime
                .apply_request_update(key.clone(), BarUpdate::confirmed(bar(600_000, 5.0)))
                .unwrap();
            runtime
                .append_bar_with_kind(bar(840_000, 1.0), BarUpdateKind::Confirmed)
                .unwrap();
            assert!(
                runtime.request_evaluations.is_empty(),
                "{expression}/confirmed"
            );
            assert_eq!(
                runtime.result().plots[0]
                    .values
                    .last()
                    .and_then(PineValue::as_f64),
                Some(expected_values[2]),
                "{expression}/confirmed"
            );
        }
    }
}
