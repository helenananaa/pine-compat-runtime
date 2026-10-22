//! Select initial datasets before exposing any chart execution to the host.
use super::ContextWindows;
use crate::builtins::args::call_arg_expr;
use crate::runtime::hir_walk;
use crate::{Bar, HistoricalRuntime, PineValue, RequestKey, RequestTimeframe, RuntimeError};
use pine_ir::{CallSiteId, HirExpr, HirExprKind, Qualifier, SymbolId};
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq)]
enum Preparation {
    Native,
    StaticOnly,
    Observed,
}

impl HistoricalRuntime<'_> {
    pub(crate) fn prepare_context_windows(
        &mut self,
        bars: &[Bar],
        execution_times: Option<&[i64]>,
    ) -> Result<usize, RuntimeError> {
        self.prepare_context_windows_policy(bars, execution_times, Preparation::Native)
    }

    pub(crate) fn prepare_context_windows_without_discovery(
        &mut self,
        bars: &[Bar],
        execution_times: Option<&[i64]>,
    ) -> Result<usize, RuntimeError> {
        self.prepare_context_windows_policy(bars, execution_times, Preparation::StaticOnly)
    }

    pub(crate) fn prepare_context_windows_for_observed_feedback(
        &mut self,
        bars: &[Bar],
    ) -> Result<usize, RuntimeError> {
        self.prepare_context_windows_policy(bars, None, Preparation::Observed)
    }

    fn prepare_context_windows_policy(
        &mut self,
        bars: &[Bar],
        execution_times: Option<&[i64]>,
        preparation: Preparation,
    ) -> Result<usize, RuntimeError> {
        if self.bars != 0 || self.context_windows.prepared {
            return Ok(0);
        }
        let chart = self.request_environment.chart();
        let mut windows = ContextWindows::new(
            RequestKey::new(chart.symbol(), chart.timeframe().clone()),
            self.program.language_version,
            self.program.dynamic_requests,
            self.program.calc_bars_count,
        );
        if bars.is_empty() || !self.metadata.context_windows.has_window_demands() {
            let skip = windows.initial_skip(bars.len());
            if preparation == Preparation::Observed {
                self.context_windows = windows;
            }
            return Ok(skip);
        }
        windows.prepared = true;
        windows.set_dataset(bars);
        let mut pending = self.metadata.context_windows.root_live.clone();
        // Static requests and simple v6 contexts are configured before Pine
        // starts. A series selector instead registers when execution reaches it.
        if !self.program.dynamic_requests || self.program.language_version.is_some_and(|v| v >= 6) {
            let mut evaluator = self.blank_for_replay();
            evaluator.current_bar = Some(bars[0]);
            evaluator.set_builtin_symbols(&bars[0], 0)?;
            for call in &self.metadata.context_windows.calls {
                if !self.metadata.context_windows.root_live.contains(&call.id) {
                    continue;
                }
                let Some(symbol) = call_arg_expr(&call.args, 0, "symbol") else {
                    continue;
                };
                let Some(timeframe) = call_arg_expr(&call.args, 1, "timeframe") else {
                    continue;
                };
                if symbol.pine_type.qualifier == Qualifier::Series
                    || timeframe.pine_type.qualifier == Qualifier::Series
                {
                    continue;
                }
                let enabled = self
                    .metadata
                    .context_windows
                    .guards
                    .get(&call.id)
                    .into_iter()
                    .flatten()
                    .all(|(guard, expected)| {
                        guard.pine_type.qualifier == Qualifier::Series
                            || evaluator
                                .window_constant(guard)
                                .is_none_or(|value| value.is_condition_true() == *expected)
                    });
                if !enabled {
                    pending.remove(&call.id);
                    continue;
                }
                let (Some(PineValue::String(symbol)), Some(PineValue::String(timeframe))) = (
                    evaluator.window_constant(symbol),
                    evaluator.window_constant(timeframe),
                ) else {
                    continue;
                };
                let chart = self.request_environment.chart();
                let symbol = if symbol.is_empty() {
                    chart.symbol()
                } else {
                    &symbol
                };
                let timeframe = if timeframe.is_empty() {
                    chart.timeframe().clone()
                } else {
                    let Ok(timeframe) = RequestTimeframe::parse(&timeframe) else {
                        continue;
                    };
                    timeframe
                };
                let count = call_arg_expr(
                    &call.args,
                    if call.callee == "request.security" {
                        7
                    } else {
                        6
                    },
                    "calc_bars_count",
                );
                let count = match count {
                    Some(expr) => match evaluator
                        .window_constant(expr)
                        .and_then(|value| value.as_i64())
                    {
                        Some(count) if count >= 0 => (count > 0).then_some(count as usize),
                        _ => continue,
                    },
                    None => None,
                };
                let key = RequestKey::new(symbol, timeframe);
                if !(windows.is_root(&key)
                    && self
                        .metadata
                        .context_windows
                        .chart_passthrough
                        .contains(&call.id))
                {
                    windows.register_call(call.id, key, count, bars);
                }
                pending.remove(&call.id);
            }
        }
        if preparation == Preparation::StaticOnly && !pending.is_empty() {
            return Err(RuntimeError { message: "external streaming dataset requires statically resolved request windows; dynamic discovery needs observed account feedback".to_owned() });
        }
        if preparation == Preparation::Observed {
            windows.discovering = !pending.is_empty();
            let skip = windows.initial_skip(bars.len());
            self.context_windows = windows;
            return Ok(skip);
        }
        // Discovery uses disposable Pine/broker/output state. Preserve context
        // reservations across restarts, never partially executed Pine state.
        while !pending.is_empty() {
            windows.restart = false;
            windows.discovering = true;
            let skip = windows.initial_skip(bars.len());
            let mut probe = self.blank_for_replay();
            probe.external_execution = self.external_execution.clone();
            probe.context_windows = windows;
            probe.prepare_external_window(skip)?;
            probe.static_requests.extend_dataset(0, &bars[skip..]);
            probe.historical_end = Some(bars.len() - skip);
            probe.last_bar_index = bars.len().checked_sub(skip + 1);
            probe.last_bar_time = bars.last().map(|bar| bar.time);
            for (offset, bar) in bars[skip..].iter().enumerate() {
                let result = match execution_times {
                    Some(times) => probe.append_bar_with_execution_time(*bar, times[skip + offset]),
                    None => probe.append_bar(*bar),
                };
                if result.is_err() {
                    break;
                }
            }
            windows = probe.context_windows;
            if !windows.restart {
                break;
            }
        }
        let skip = windows.initial_skip(bars.len());
        windows.finish();
        self.context_windows = windows;
        Ok(skip)
    }

    fn window_constant(&mut self, expr: &HirExpr) -> Option<PineValue> {
        if expr.pine_type.qualifier == Qualifier::Series {
            return None;
        }
        self.window_dependencies(expr, &mut HashSet::new())?;
        self.eval_expr(expr).ok()
    }

    fn window_dependencies(
        &mut self,
        expr: &HirExpr,
        visiting: &mut HashSet<SymbolId>,
    ) -> Option<()> {
        let mut symbols = Vec::new();
        hir_walk::expression(
            expr,
            &mut |expr| {
                if let HirExprKind::Symbol(symbol) = expr.kind {
                    symbols.push(symbol);
                }
            },
            &mut |_| {},
        );
        for symbol in symbols {
            if self.current_symbols.contains_key(&symbol) {
                continue;
            }
            if !visiting.insert(symbol) {
                return None;
            }
            let initializer = self
                .metadata
                .context_windows
                .initializers
                .get(&symbol)?
                .clone();
            if initializer.pine_type.qualifier == Qualifier::Series {
                return None;
            }
            self.window_dependencies(&initializer, visiting)?;
            let value = self.eval_expr(&initializer).ok()?;
            self.current_symbols.insert(symbol, value);
            visiting.remove(&symbol);
        }
        Some(())
    }

    /// Called after selectors are evaluated, before expression evaluation.
    pub(crate) fn register_context_window(
        &mut self,
        id: CallSiteId,
        key: RequestKey,
        count: Option<usize>,
    ) -> Result<bool, RuntimeError> {
        if !self.context_windows.prepared {
            return Ok(true);
        }
        if !self.metadata.context_windows.live.contains(&id) {
            return Ok(false);
        }
        if self.context_windows.is_root(&key)
            && self
                .metadata
                .context_windows
                .chart_passthrough
                .contains(&id)
        {
            return Ok(true);
        }
        self.context_windows
            .register_call(id, key.clone(), count, &[]);
        if let Some(times) = self.request_window_history.get(&key) {
            self.context_windows.anchor_known_history(&key, times);
        }
        if self.context_windows.restart {
            // Native discovery consumes this internally. Observed discovery
            // exposes a typed control status and requires a complete rebase.
            return Err(RuntimeError {
                message: "initial context window changed".to_owned(),
            });
        }
        Ok(true)
    }
}
