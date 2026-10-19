use crate::{Bar, BarUpdateKind, CurrencyRateError, HistoricalRuntime, RuntimeError};

impl HistoricalRuntime<'_> {
    pub(crate) fn account_currency_rate_for_bar(
        &self,
        bar: &Bar,
        _kind: BarUpdateKind,
        _execution_time: Option<i64>,
    ) -> Result<Option<f64>, RuntimeError> {
        let chart = self.request_environment.chart().currency();
        let account = self
            .program
            .strategy_settings
            .account_currency
            .unwrap_or(chart);
        if account == chart {
            return Ok(Some(1.0));
        }
        // Native historical and realtime D/W/M probes use the effective daily
        // value at chart-bar open, even when timenow is several days later.
        match self
            .request_environment
            .currency_rate(chart, account, bar.time)
        {
            Ok(rate) => Ok(Some(rate)),
            Err(CurrencyRateError::MissingHistory { .. }) => Ok(None),
            Err(error) => Err(RuntimeError {
                message: format!(
                    "strategy account currency {account} differs from chart currency {chart}: {error}"
                ),
            }),
        }
    }

    pub(crate) fn current_account_currency_rate(&self) -> Result<Option<f64>, RuntimeError> {
        let bar = self.current_bar.as_ref().ok_or_else(|| RuntimeError {
            message: "currency conversion requires an active bar".to_owned(),
        })?;
        self.account_currency_rate_for_bar(
            bar,
            self.current_bar_update_kind,
            self.current_execution_time,
        )
    }
}
