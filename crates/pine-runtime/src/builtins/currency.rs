use super::args::call_arg_expr;
use crate::{CurrencyRateError, HistoricalRuntime, PineValue, RuntimeError};
use pine_ir::HirCallArg;

impl HistoricalRuntime<'_> {
    pub(super) fn eval_currency_rate(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let from = match call_arg_expr(args, 0, "from") {
            Some(expr) => self.eval_expr(expr)?,
            None => PineValue::Na,
        };
        let to = match call_arg_expr(args, 1, "to") {
            Some(expr) => self.eval_expr(expr)?,
            None => PineValue::Na,
        };
        let ignore = match call_arg_expr(args, 2, "ignore_invalid_currency") {
            Some(expr) => matches!(self.eval_expr(expr)?, PineValue::Bool(true)),
            None => false,
        };
        let (PineValue::String(from), PineValue::String(to)) = (from, to) else {
            if ignore {
                return Ok(PineValue::Na);
            }
            return Err(RuntimeError {
                message: "request.currency_rate requires valid from/to currency strings".to_owned(),
            });
        };
        // Native accepts lowercase ISO names; provider keys stay canonical so
        // a host need not register duplicate streams for spelling variants.
        let from = from.to_ascii_uppercase();
        let to = to.to_ascii_uppercase();
        let time = if from == to {
            0
        } else {
            let bar = self.current_bar.as_ref().ok_or_else(|| RuntimeError {
                message: "request.currency_rate requires an active bar".to_owned(),
            })?;
            bar.time
        };
        match self.request_environment.currency_rate(&from, &to, time) {
            Ok(rate) => Ok(PineValue::Float(rate)),
            Err(CurrencyRateError::MissingHistory { .. }) => Ok(PineValue::Na),
            Err(
                CurrencyRateError::UnavailablePair { .. } | CurrencyRateError::InvalidCurrency(_),
            ) if ignore => Ok(PineValue::Na),
            Err(error) => Err(RuntimeError {
                message: error.to_string(),
            }),
        }
    }
}
