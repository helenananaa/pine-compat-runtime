use pine_ir::{HirCallArg, StrategyDefaultQuantity};

use crate::builtins::args::call_arg_expr;
use crate::{HistoricalRuntime, PineValue, RuntimeError};

impl HistoricalRuntime<'_> {
    pub(super) fn default_order_qty(
        &self,
        equity: f64,
        close: f64,
        is_long: bool,
        market: bool,
    ) -> Result<Option<f64>, RuntimeError> {
        // Cash market sizing includes expected slippage, while the public
        // query uses the caller's fill price without a second adjustment.
        let price = if market
            && matches!(
                self.program.strategy_settings.default_qty,
                Some(StrategyDefaultQuantity::Cash(_))
            ) {
            let slippage = self.program.strategy_settings.slippage_ticks
                * self.request_environment.chart().min_tick();
            close + if is_long { slippage } else { -slippage }
        } else {
            close
        };
        let price = price * self.strategy_broker.quote_to_account_rate();
        let scale = self.request_environment.chart().configured_quantity_scale();
        let qty = match self.program.strategy_settings.default_qty {
            Some(StrategyDefaultQuantity::PercentOfEquity(percent))
                if equity.is_finite() && price.is_finite() && price != 0.0 =>
            {
                // Native zero budgets are legal. Negative fill prices can
                // produce signed query results even with positive equity.
                // The negative-equity error applies only to a negative budget
                // divided by a positive price, before contract-grid rounding.
                if equity < 0.0 && percent > 0.0 && price > 0.0 {
                    return Err(RuntimeError {
                        message: format!(
                            "Cannot create an order with negative quantity on bar {}: current qty_type is percent_of_equity and equity is less than 0",
                            self.bars
                        ),
                    });
                }
                Some(crate::strategy::percent_of_equity_order_qty(
                    equity,
                    percent,
                    price,
                    self.program.strategy_settings.commission,
                    scale,
                ))
            }
            _ => self
                .program
                .strategy_settings
                .default_entry_qty(equity, price)
                .map(|qty| crate::strategy::quantity_on_chart_grid(qty, scale)),
        };
        Ok(qty)
    }

    pub(super) fn eval_strategy_default_entry_qty(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let Some(fill_price_expr) = call_arg_expr(args, 0, "fill_price") else {
            return Ok(PineValue::Na);
        };
        let fill_price = self
            .eval_expr(fill_price_expr)?
            .as_f64()
            .unwrap_or(f64::NAN);
        let Some(bar) = self.current_bar else {
            return Err(RuntimeError {
                message: "`strategy.default_entry_qty` requires an active bar".to_owned(),
            });
        };
        // Native cash/percent queries return zero for missing or zero prices,
        // before checking equity. Fixed quantities do not depend on price.
        if matches!(
            self.program.strategy_settings.default_qty,
            Some(StrategyDefaultQuantity::Cash(_) | StrategyDefaultQuantity::PercentOfEquity(_))
        ) && (!fill_price.is_finite() || fill_price == 0.0)
        {
            return Ok(PineValue::Float(0.0));
        }
        let equity = self.strategy_broker.equity_value(bar.close);
        Ok(self
            .default_order_qty(equity, fill_price, true, false)?
            .map_or(PineValue::Na, PineValue::Float))
    }
}
