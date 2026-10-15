use super::*;
use crate::strategy::{CombinedExitSpec, ExitQuantityRequest};

impl HistoricalRuntime<'_> {
    pub(super) fn eval_combined_exit(
        &mut self,
        args: &[HirCallArg],
        id: String,
        from_entry: String,
        quantity: StrategyExitQuantityArg,
        metadata: StrategyExitMetadata,
    ) -> Result<PineValue, RuntimeError> {
        let mut value = |index, name| -> Result<Option<f64>, RuntimeError> {
            match call_arg_expr(args, index, name) {
                Some(expr) => Ok(self
                    .eval_expr(expr)?
                    .as_f64()
                    .filter(|value| value.is_finite())),
                None => Ok(None),
            }
        };
        let spec = CombinedExitSpec {
            stop: value(2, "stop")?,
            limit: value(3, "limit")?,
            profit: value(4, "profit")?,
            loss: value(5, "loss")?,
            trail_price: value(6, "trail_price")?,
            trail_points: value(7, "trail_points")?,
            trail_offset: value(8, "trail_offset")?,
            mintick: self.request_environment.chart().min_tick(),
            compare_relative: self.program.language_version.is_some_and(|v| v >= 6),
        };
        let quantity = match quantity {
            StrategyExitQuantityArg::Full => ExitQuantityRequest::Full,
            StrategyExitQuantityArg::Fixed(qty) => ExitQuantityRequest::Fixed(qty),
            StrategyExitQuantityArg::Percent(qty) => ExitQuantityRequest::Percent(qty),
        };
        self.strategy_broker
            .place_combined_exit(id, from_entry, spec, quantity, self.bars, metadata);
        Ok(PineValue::Void)
    }
}
