//! Static requests consume independent chart-context request series. Their
//! expressions are not re-evaluated in the enclosing request's context.
use std::collections::{HashMap, HashSet};

use pine_ir::{CallSiteId, HirCallArg, HirExpr, HirExprKind, HirProgram, SymbolId};

use super::request_values::RequestedValue;
use super::requests::{
    RequestMergeClock, RequestMergePolicy, align_requested_value_with_close,
    request_capture_values, request_dependency_initializers, request_tuple_dependency_statements,
};
use crate::runtime::{append_history::AppendHistory, hir_walk};
use crate::{Bar, BarUpdateKind, HistoricalRuntime, PineValue, RequestEnvironment, RuntimeError};

#[derive(Clone, Default)]
pub(crate) struct StaticRequests {
    environment: Option<RequestEnvironment>,
    chart_bars: AppendHistory<Bar>,
    requested: bool,
    root_update_kind: Option<BarUpdateKind>,
    stack: Vec<CallSiteId>,
    values: HashMap<CallSiteId, AppendHistory<(i64, RequestedValue)>>,
}

impl StaticRequests {
    pub(crate) fn new(enabled: bool, environment: RequestEnvironment) -> Self {
        Self {
            environment: enabled.then_some(environment),
            ..Self::default()
        }
    }

    pub(crate) fn for_requested_context(&self) -> Self {
        Self {
            requested: true,
            ..self.clone()
        }
    }

    pub(crate) fn is_requested_context(&self) -> bool {
        self.requested && self.environment.is_some()
    }

    pub(crate) fn chart_history_profile(&self) -> (usize, usize) {
        (self.chart_bars.len(), self.chart_bars.capacity())
    }

    pub(crate) fn extend_dataset(&mut self, start: usize, bars: &[Bar]) -> bool {
        if self.environment.is_some() && !self.requested {
            self.chart_bars.truncate(start);
            for bar in bars {
                self.chart_bars.push(*bar);
            }
            self.values.clear();
            return true;
        }
        false
    }

    pub(crate) fn observe_bar(&mut self, index: usize, bar: Bar, kind: BarUpdateKind) -> bool {
        if self.environment.is_some() && !self.requested {
            self.root_update_kind = Some(kind);
        }
        if self.environment.is_some() && !self.requested && index == self.chart_bars.len() {
            self.chart_bars.push(bar);
            self.values.clear();
            return true;
        }
        false
    }

    pub(crate) fn finish_dataset(&mut self, committed: usize) -> bool {
        if !self.requested && self.chart_bars.len() > committed {
            self.chart_bars.truncate(committed);
            self.values.clear();
            return true;
        }
        false
    }
}

pub(crate) fn is_security(name: &str) -> bool {
    name == "request.security" || name.starts_with("$legacy.security.")
}

pub(crate) fn needs_chart_history(program: &HirProgram) -> bool {
    if program.dynamic_requests {
        return false;
    }
    let mut initializers = request_dependency_initializers(program);
    for (symbol, statement) in request_tuple_dependency_statements(program) {
        if let pine_ir::HirStmtKind::TupleDecl { value, .. } = &statement.kind {
            initializers.entry(symbol).or_insert(value);
        }
    }
    let mut needed = false;
    hir_walk::statements(&program.statements, &mut |expr| {
        if let HirExprKind::Call { callee, args, .. } = &expr.kind
            && is_security(callee)
            && let Some(expression) = crate::builtins::args::call_arg_expr(args, 2, "expression")
        {
            needed |= contains_request(expression, &initializers, &mut HashSet::new());
        }
    });
    needed
}

fn contains_request(
    expression: &HirExpr,
    initializers: &HashMap<SymbolId, &HirExpr>,
    visited: &mut HashSet<SymbolId>,
) -> bool {
    let mut pending = vec![expression];
    let mut found = false;
    while let Some(expression) = pending.pop() {
        hir_walk::expression(
            expression,
            &mut |expr| match &expr.kind {
                HirExprKind::Call { callee, .. } if is_security(callee) => found = true,
                HirExprKind::Symbol(symbol) if visited.insert(*symbol) => {
                    if let Some(initializer) = initializers.get(symbol) {
                        pending.push(*initializer);
                    }
                }
                _ => {}
            },
            &mut |_| {},
        );
    }
    found
}

impl HistoricalRuntime<'_> {
    pub(crate) fn eval_static_request(
        &mut self,
        callee: &str,
        site: CallSiteId,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let environment = self
            .static_requests
            .environment
            .clone()
            .ok_or_else(|| RuntimeError {
                message: "static request dependency has no chart history".to_owned(),
            })?;
        if self.static_requests.stack.len() >= 64 || self.static_requests.stack.contains(&site) {
            return Err(RuntimeError {
                message: "recursive static request dependency".to_owned(),
            });
        }
        if !self.static_requests.values.contains_key(&site) {
            let program = self.program.clone();
            let initializers = request_dependency_initializers(&program);
            let tuples = request_tuple_dependency_statements(&program);
            let expression_type = crate::builtins::args::call_arg_expr(args, 2, "expression")
                .ok_or_else(|| RuntimeError {
                    message: "static request has no expression".to_owned(),
                })?
                .pine_type;
            let call = HirExpr {
                kind: HirExprKind::Call {
                    callee: callee.to_owned(),
                    call_site_id: site,
                    args: args.to_vec(),
                },
                pine_type: expression_type,
                series_id: None,
            };
            let captures = request_capture_values(
                &program,
                &call,
                &self.current_symbols,
                &initializers,
                &tuples,
            );
            let mut root = self.fork_with_request_environment(environment.clone());
            root.static_requests.requested = false;
            root.static_requests.stack.push(site);
            root.static_requests.values.clear();
            let bars = self.static_requests.chart_bars.clone();
            root.historical_end = Some(bars.len());
            root.inherit_valuewhen_budget(self, 0)?;
            let mut values = AppendHistory::default();
            for (index, bar) in bars.iter().enumerate() {
                let kind = if index + 1 == bars.len() {
                    self.static_requests
                        .root_update_kind
                        .unwrap_or(BarUpdateKind::Historical)
                } else {
                    BarUpdateKind::Historical
                };
                let result = root.eval_requested_bar_expression_with_kind(
                    *bar,
                    &call,
                    &captures,
                    &initializers,
                    &tuples,
                    kind,
                );
                self.accept_execution_budget(&root);
                values.push((bar.time, result?));
            }
            self.legacy_security_repaint_warnings
                .extend(root.legacy_security_repaint_warnings);
            self.static_requests.values.insert(site, values);
        }
        let time = self
            .current_bar
            .ok_or_else(|| RuntimeError {
                message: "static request dependency has no current requested bar".to_owned(),
            })?
            .time;
        let chart = self.request_environment.chart();
        let key = crate::RequestKey::new(chart.symbol(), chart.timeframe().clone());
        let nominal_close = self.request_environment.chart_close(time)?;
        // A supplied successor can end a session's last requested bar before
        // its nominal duration (e.g. 11-minute crypto bars at UTC midnight).
        let next_open = self
            .request_environment
            .provider()
            .bars(&key)
            .ok()
            .and_then(|bars| bars.get(bars.partition_point(|bar| bar.time <= time)))
            .map(|bar| bar.time);
        let chart_close = next_open.map_or(nominal_close, |next| nominal_close.min(next));
        // Static dependencies are already computed chart series. Sample the
        // last completed chart value at this request's close, rather than
        // applying security's lower-timeframe intrabar selection to them.
        let root_key = crate::RequestKey::new(
            environment.chart().symbol(),
            environment.chart().timeframe().clone(),
        );
        let value = align_requested_value_with_close(
            &self.static_requests.values[&site],
            time,
            environment.chart().timeframe(),
            self.request_environment.chart().timeframe(),
            RequestMergePolicy::MODERN,
            BarUpdateKind::Historical,
            false,
            chart_close,
            RequestMergeClock::new(&self.request_environment, &root_key),
        )?;
        Ok(self.import_requested_value(&value))
    }
}
