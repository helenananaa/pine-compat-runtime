use super::BrokerState;
use crate::CurrencyRateError;

impl BrokerState {
    /// Set units of account currency per unit of the chart's quote currency.
    /// Prices and quantities remain in their original instrument units.
    pub fn set_quote_to_account_rate(&mut self, rate: f64) -> Result<(), CurrencyRateError> {
        if !rate.is_finite() || rate <= 0.0 {
            return Err(CurrencyRateError::InvalidResolvedRate);
        }
        if rate != self.quote_to_account_rate {
            // Pine converts price-change P/L, rather than treating the entire
            // position notional as a foreign cash asset. Rebase the accounting
            // cash leg so an unchanged instrument price cannot create FX P/L.
            self.cash = self.initial_capital + self.net_profit()
                - self.position_size * self.avg_price * self.point_value * rate;
            self.quote_to_account_rate = rate;
        }
        Ok(())
    }

    pub(crate) fn quote_to_account_rate(&self) -> f64 {
        self.quote_to_account_rate
    }

    /// Account currency represented by one price point of one contract.
    pub(super) fn contract_point_value_to_account(&self) -> f64 {
        self.point_value * self.quote_to_account_rate
    }

    pub(super) fn contract_value(&self, price_quantity: f64) -> f64 {
        price_quantity * self.contract_point_value_to_account()
    }
}
