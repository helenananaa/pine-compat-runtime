//! Native historical observations. Broker transitions happen between script
//! observations; a fill does not add a second execution at the same vertex.
use super::*;
use crate::runtime::strategy_path::{
    HistoricalPath, MagnifierHostGap, PathLeg, magnifier_host_sequence,
};
use crate::strategy::EntryPathTick;

impl HistoricalRuntime<'_> {
    pub(super) fn run_native_history_ticks(
        &mut self,
        index: usize,
        bar: Bar,
    ) -> Result<(), RuntimeError> {
        self.historical_tick_context_active = true;
        let sequence = magnifier_host_sequence(
            index,
            bar,
            &self.magnifier_input,
            self.program.strategy_settings.use_bar_magnifier,
        );
        if let Some(warning) = sequence.warning {
            self.push_magnifier_diagnostic(warning);
        }
        let opening = sequence.bars.first().map_or(bar.open, |host| host.bar.open);
        self.strategy_broker.reset_risk_windows(
            index,
            bar.time,
            crate::builtins::time::timeframe_seconds(crate::DEFAULT_CHART_TIMEFRAME).unwrap_or(0),
            self.strategy_broker.equity_value(opening),
            opening,
            self.session_windows.ids_for(index),
        );
        self.strategy_broker
            .fill_pending_market_closes(index, bar.time, opening);
        self.strategy_broker
            .fill_pending_market_entries(index, bar.time, opening);
        let mut visible = Bar {
            high: bar.open,
            low: bar.open,
            close: bar.open,
            volume: 0.0,
            ..bar
        };
        let mut prior_volume = 0.0;
        let mut previous = self
            .strategy_scheduler
            .last_host_bar()
            .map(|previous| previous.close);
        for (host_index, host) in sequence.bars.iter().enumerate() {
            let Some(path) = HistoricalPath::from_validated_bar(&host.bar) else {
                return Err(RuntimeError {
                    message: "invalid history-tick OHLC path".to_owned(),
                });
            };
            if let Some(previous_close) = previous {
                self.consume_history_tick_gap(index, bar.time, previous_close, host.bar.open)?;
            }
            for (tick, point) in path.points.iter().enumerate() {
                if tick > 0 {
                    let leg = path.legs()[tick - 1];
                    let mut mark = leg.from.price;
                    let mut exhausted = true;
                    for _ in 0..10_000 {
                        let event =
                            self.strategy_broker
                                .take_next_entry_path_event(EntryPathTick {
                                    bar_index: index,
                                    time: bar.time,
                                    leg,
                                    path_kind: path.kind,
                                    mark,
                                    long_blocked_at_path_start: self
                                        .strategy_broker
                                        .same_side_long_entry_blocked(),
                                    short_blocked_at_path_start: self
                                        .strategy_broker
                                        .same_side_short_entry_blocked(),
                                });
                        let Some(event) = event else {
                            exhausted = false;
                            break;
                        };
                        mark = event.mark();
                    }
                    if exhausted {
                        return Err(history_tick_progress_error());
                    }
                }
                visible.high = visible.high.max(point.price);
                visible.low = visible.low.min(point.price);
                visible.close = point.price;
                visible.volume = prior_volume + host.bar.volume * (tick + 1) as f64 / 4.0;
                self.current_bar = Some(visible);
                self.observe_history_tick_account(index, bar.time, point.price);
                self.run_strategy_script_pass()?;
                let final_tick = host_index + 1 == sequence.bars.len() && tick == 3;
                if !final_tick || self.program.strategy_settings.process_orders_on_close {
                    self.strategy_broker
                        .fill_same_bar_market_closes(index, bar.time, point.price);
                    self.strategy_broker
                        .fill_same_bar_market_entries(index, bar.time, point.price);
                    if final_tick {
                        self.strategy_broker.fill_same_bar_limit_entries(
                            index,
                            bar.time,
                            point.price,
                        );
                        self.strategy_broker.fill_same_bar_close_exits(
                            index,
                            bar.time,
                            point.price,
                        );
                    }
                }
                self.observe_history_tick_account(index, bar.time, point.price);
            }
            prior_volume += host.bar.volume;
            previous = Some(host.bar.close);
        }
        self.strategy_scheduler
            .set_last_host_bar(sequence.bars.last().map(|host| host.bar));
        self.current_bar = Some(bar);
        self.historical_tick_context_active = false;
        self.strategy_broker.record_equity(index, bar.close);
        Ok(())
    }

    fn consume_history_tick_gap(
        &mut self,
        index: usize,
        time: i64,
        previous_close: f64,
        next_open: f64,
    ) -> Result<(), RuntimeError> {
        let tick = EntryPathTick {
            bar_index: index,
            time,
            leg: PathLeg::point(next_open),
            path_kind: crate::runtime::strategy_path::HistoricalPathKind::OpenHighLowClose,
            mark: next_open,
            long_blocked_at_path_start: self.strategy_broker.same_side_long_entry_blocked(),
            short_blocked_at_path_start: self.strategy_broker.same_side_short_entry_blocked(),
        };
        let gap = MagnifierHostGap {
            previous_close,
            next_open,
        };
        for _ in 0..10_000 {
            if self
                .strategy_broker
                .take_next_gap_event(tick, gap)
                .is_none()
            {
                return Ok(());
            }
        }
        Err(history_tick_progress_error())
    }

    fn observe_history_tick_account(&mut self, index: usize, time: i64, mark: f64) {
        self.strategy_broker.update_open_trade_extremes(mark, mark);
        self.strategy_broker
            .evaluate_risk_equity_stops(index, time, mark);
        self.strategy_broker
            .evaluate_margin_call_long(index, time, mark);
        self.strategy_broker
            .evaluate_margin_call_short(index, time, mark);
        self.strategy_broker
            .flatten_if_risk_blocked(index, time, mark);
    }
}

fn history_tick_progress_error() -> RuntimeError {
    RuntimeError {
        message: "strategy history-tick event loop made no progress".to_owned(),
    }
}
