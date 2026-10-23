/// Account-currency extrema per remaining contract, frozen at observation time.
/// The entry fee stays in account units when the conversion rate changes.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct TradeExtrema {
    pub(super) runup: f64,
    pub(super) drawdown: f64,
    pub(super) runup_percent: f64,
    pub(super) drawdown_percent: f64,
}

impl TradeExtrema {
    pub(super) fn observe(
        &mut self,
        entry_price: f64,
        signed_unit: f64,
        entry_fee_per_unit: f64,
        range: (f64, f64),
        rate: f64,
    ) {
        let denominator = entry_price * rate + entry_fee_per_unit;
        for price in [range.0, range.1] {
            if !price.is_finite() {
                continue;
            }
            let profit = (price - entry_price) * signed_unit * rate - entry_fee_per_unit;
            self.runup = self.runup.max(profit);
            self.drawdown = self.drawdown.max(-profit);
            if denominator.is_finite() && denominator > 0.0 {
                self.runup_percent = self.runup_percent.max(profit / denominator * 100.0);
                self.drawdown_percent = self.drawdown_percent.max(-profit / denominator * 100.0);
            }
        }
        for metric in [
            &mut self.runup,
            &mut self.drawdown,
            &mut self.runup_percent,
            &mut self.drawdown_percent,
        ] {
            if *metric == 0.0 {
                *metric = 0.0;
            }
        }
    }
}
