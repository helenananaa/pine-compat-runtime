use super::BrokerState;

impl BrokerState {
    pub(super) fn observe_strategy_equity_extremes(&mut self, mark: f64) {
        if !mark.is_finite() {
            return;
        }
        let equity = self.equity_value(mark);
        let runup = (equity - self.min_equity_before_open_trade).max(0.0);
        let drawdown = (self.max_equity_before_open_trade - equity).max(0.0);
        self.max_runup = self.max_runup.max(runup);
        self.max_drawdown = self.max_drawdown.max(drawdown);
        // Native percentages use account equity at the observed run-up and
        // the closed-equity peak preceding the position for drawdown.
        if equity.is_finite() && equity > 0.0 {
            self.max_runup_percent = self.max_runup_percent.max(runup / equity * 100.0);
        }
        if self.max_equity_before_open_trade.is_finite() && self.max_equity_before_open_trade > 0.0
        {
            self.max_drawdown_percent = self
                .max_drawdown_percent
                .max(drawdown / self.max_equity_before_open_trade * 100.0);
        }
    }
}
