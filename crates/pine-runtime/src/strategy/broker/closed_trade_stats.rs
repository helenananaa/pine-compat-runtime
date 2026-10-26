//! Running totals behind the closed-trade statistic variables.
//!
//! Closed trades are append-only, so every close folds into these totals once
//! and in trade order: the same left fold a scan of the closed-trade history
//! performs. Each `strategy.wintrades`, `strategy.grossprofit` or average read
//! is therefore constant time without changing its value. Broker checkpoints
//! clone the totals together with the trade history they summarize.

/// Totals for one trade outcome. Losing amounts and percentages are stored as
/// magnitudes, matching `strategy.grossloss` and the losing-trade averages.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct OutcomeTotals {
    count: usize,
    amount: f64,
    percent: f64,
}

impl OutcomeTotals {
    fn record(&mut self, amount: f64, percent: f64) {
        self.count += 1;
        self.amount += amount;
        self.percent += percent;
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct ClosedTradeStats {
    count: usize,
    percent: f64,
    winning: OutcomeTotals,
    losing: OutcomeTotals,
    even_count: usize,
}

impl ClosedTradeStats {
    pub(super) fn record(&mut self, profit: f64, profit_percent: f64) {
        self.count += 1;
        self.percent += profit_percent;
        if profit > 0.0 {
            self.winning.record(profit, profit_percent);
        } else if profit < 0.0 {
            self.losing.record(-profit, -profit_percent);
        } else if profit == 0.0 {
            // Both signed zeros are even; NaN counts toward no outcome.
            self.even_count += 1;
        }
    }

    pub(super) fn winning_count(&self) -> usize {
        self.winning.count
    }

    pub(super) fn losing_count(&self) -> usize {
        self.losing.count
    }

    pub(super) fn even_count(&self) -> usize {
        self.even_count
    }

    pub(super) fn gross_profit(&self) -> f64 {
        self.winning.amount
    }

    pub(super) fn gross_loss(&self) -> f64 {
        self.losing.amount
    }

    pub(super) fn average_winning_trade(&self) -> Option<f64> {
        average(self.winning.amount, self.winning.count)
    }

    pub(super) fn average_losing_trade(&self) -> Option<f64> {
        average(self.losing.amount, self.losing.count)
    }

    pub(super) fn average_trade_percent(&self) -> Option<f64> {
        average(self.percent, self.count)
    }

    pub(super) fn average_winning_trade_percent(&self) -> Option<f64> {
        average(self.winning.percent, self.winning.count)
    }

    pub(super) fn average_losing_trade_percent(&self) -> Option<f64> {
        average(self.losing.percent, self.losing.count)
    }
}

fn average(total: f64, count: usize) -> Option<f64> {
    (count > 0).then(|| total / count as f64)
}
