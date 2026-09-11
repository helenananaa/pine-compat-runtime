//! Reuse a checkpoint preceding the terminal requested bar. The old terminal
//! bar is always replayed when a context grows, preserving endpoint semantics.
use super::requests::{request_capture_values, request_dependency_initializers};
use crate::request::RequestCacheKey;
use crate::runtime::append_history::AppendHistory;
use crate::{
    HistoricalRuntime, PineValue, RequestDataError, RequestEnvironment, RequestKey, RuntimeError,
};
use pine_ir::{HirExpr, HirExprKind, HirHistoryOffset, SymbolId};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub(crate) struct RequestEvaluation<'a> {
    prefix: AppendHistory<(i64, PineValue)>,
    before_last: Arc<HistoricalRuntime<'a>>,
    captures: HashMap<SymbolId, PineValue>,
    provider_len: usize,
    #[cfg(test)]
    replayed_bars: usize,
}

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn evaluate_request_incremental(
        &mut self,
        key: &RequestKey,
        cache_key: &RequestCacheKey,
        expression: &HirExpr,
        environment: RequestEnvironment,
        include_forming: bool,
    ) -> Result<AppendHistory<(i64, PineValue)>, RuntimeError> {
        let initializers = request_dependency_initializers(&self.program);
        let captures = request_capture_values(
            &self.program,
            expression,
            &self.current_symbols,
            &initializers,
        );
        if !incremental_expression(
            expression,
            &initializers,
            &captures,
            &self.program.symbols,
            0,
        ) {
            let bars = self.resolved_request_bars_from(key, 0)?;
            return self
                .evaluate_requested_values(&bars, expression, environment)
                .map(AppendHistory::from_values);
        }
        let provider_len = match environment.provider().bars(key) {
            Ok(bars) => bars.len(),
            Err(RequestDataError::MissingData { .. }) => 0,
            Err(error) => {
                return Err(RuntimeError {
                    message: error.to_string(),
                });
            }
        };
        let count = self
            .request_feed
            .resolved_len(key, provider_len, include_forming);
        let previous = self
            .request_evaluations
            .get(cache_key)
            .filter(|saved| {
                saved.provider_len == provider_len
                    && saved.captures == captures
                    && saved.prefix.len() < count
            })
            .cloned();
        let mut prefix = previous
            .as_ref()
            .map_or_else(AppendHistory::default, |saved| saved.prefix.clone());
        let mut runtime = match previous {
            Some(saved) => (*saved.before_last).clone(),
            None => self.fork_with_request_environment(environment),
        };
        let bars = self.resolved_request_bars_from(key, prefix.len())?;
        runtime.historical_end = Some(count);
        let mut before_last = None;
        let mut last = None;
        for (index, bar) in bars.iter().enumerate() {
            let terminal = index + 1 == bars.len();
            if terminal {
                before_last = Some(Arc::new(runtime.clone()));
            }
            let value = runtime.eval_requested_bar_expression(
                *bar,
                expression,
                &captures,
                &initializers,
            )?;
            if terminal {
                last = Some((bar.time, value));
            } else {
                prefix.push((bar.time, value));
            }
        }
        let mut values = prefix.clone();
        values.push(last.expect("resolved bars are nonempty"));
        self.request_evaluations.insert(
            cache_key.clone(),
            Arc::new(RequestEvaluation {
                prefix,
                before_last: before_last.expect("terminal checkpoint"),
                captures,
                provider_len,
                #[cfg(test)]
                replayed_bars: bars.len(),
            }),
        );
        Ok(values)
    }
}

// Admission is unchanged. Complex/nested and dataset-end-dependent expressions
// retain the complete evaluator; only this optimization is conservative.
fn incremental_expression(
    expr: &HirExpr,
    initializers: &HashMap<SymbolId, &HirExpr>,
    captures: &HashMap<SymbolId, PineValue>,
    symbols: &[pine_ir::HirSymbol],
    depth: usize,
) -> bool {
    if depth > 64 {
        return false;
    }
    let visit = |expr| incremental_expression(expr, initializers, captures, symbols, depth + 1);
    match &expr.kind {
        HirExprKind::Literal(_) => true,
        HirExprKind::Builtin(name) => stable_builtin(name),
        HirExprKind::Symbol(symbol) => {
            captures.contains_key(symbol)
                || initializers.get(symbol).map_or_else(
                    || {
                        symbols
                            .iter()
                            .find(|item| item.id == *symbol)
                            .is_some_and(|item| stable_builtin(&item.name))
                    },
                    |value| visit(value),
                )
        }
        HirExprKind::Unary { expr, .. } => visit(expr),
        HirExprKind::Binary { left, right, .. } => visit(left) && visit(right),
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => visit(condition) && visit(then_expr) && visit(else_expr),
        HirExprKind::Tuple(items) => items.iter().all(visit),
        HirExprKind::History { expr, offset } => {
            visit(expr)
                && match offset {
                    HirHistoryOffset::Constant(_) => true,
                    HirHistoryOffset::Dynamic(value) => visit(value),
                }
        }
        HirExprKind::Call { callee, args, .. } => {
            (callee.starts_with("ta.") || (callee.starts_with("math.") && callee != "math.random"))
                && args.iter().all(|arg| visit(&arg.value))
        }
        _ => false,
    }
}

fn stable_builtin(name: &str) -> bool {
    matches!(
        name,
        "open"
            | "high"
            | "low"
            | "close"
            | "volume"
            | "time"
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
    fn live_ta_checkpoint_matches_forced_full_evaluation_with_bounded_replay() {
        for expression in [
            "ta.sma(close, 3)",
            "ta.ema(close, 5)",
            "ta.rsi(close, 7)",
            "ta.cum(close)",
        ] {
            let source = format!(
                "//@version=6\nindicator(\"cache\")\nplot(request.security(\"B\", \"5\", {expression}))"
            );
            let hir = analyze_source(&SourceFile::new("cache.pine", source))
                .hir
                .unwrap();
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
            fast.append_bar(bar(0, 1.0)).unwrap();
            full.append_bar(bar(0, 1.0)).unwrap();
            for index in 128..144 {
                for bump in [0.0, 0.5, 1.0] {
                    let requested = bar(index * 300_000, index as f64 + bump);
                    fast.apply_request_update(key.clone(), BarUpdate::forming(requested))
                        .unwrap();
                    full.apply_request_update(key.clone(), BarUpdate::forming(requested))
                        .unwrap();
                    full.request_evaluations.clear();
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
}
