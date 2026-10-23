//! Pine quantity selection using the authoritative account of the active pass.
use crate::builtins::args::call_arg_expr;
use crate::{HistoricalRuntime, RuntimeError};
use pine_ir::HirCallArg;

use super::external_error;

impl HistoricalRuntime<'_> {
    pub(crate) fn external_sizing_equity(&self) -> Result<f64, RuntimeError> {
        self.external_execution
            .as_ref()
            .and_then(|state| {
                state
                    .active_pass
                    .as_ref()
                    .map(|(_, pass)| &pass.account)
                    .or_else(|| state.frames.get(self.bars))
            })
            .map(|frame| frame.equity)
            .ok_or_else(|| external_error("missing account feedback for quantity sizing"))
    }

    pub(super) fn external_order_quantity(
        &mut self,
        args: &[HirCallArg],
        is_long: bool,
    ) -> Result<f64, RuntimeError> {
        let explicit = match call_arg_expr(args, 2, "qty") {
            Some(expr) => {
                let value = self.eval_expr(expr)?;
                if value.is_na() {
                    None
                } else {
                    Some(
                        value
                            .as_f64()
                            .ok_or_else(|| external_error("quantity must be numeric"))?,
                    )
                }
            }
            None => None,
        };
        let quantity = if let Some(quantity) = explicit {
            // Validate before truncation: a negative sub-contract value must
            // not become a zero no-op merely because of its quantity grid.
            if !quantity.is_finite() || quantity < 0.0 {
                return Err(external_error("quantity must be positive and finite"));
            }
            crate::strategy::explicit_quantity_on_chart_grid(
                quantity,
                self.request_environment.chart().quantity_rules(),
            )
        } else {
            let bar = self
                .current_bar
                .ok_or_else(|| external_error("quantity sizing requires an active bar"))?;
            let equity = self.external_sizing_equity()?;
            self.default_order_qty(
                equity,
                bar.close,
                is_long,
                call_arg_expr(args, 3, "limit").is_none()
                    && call_arg_expr(args, 4, "stop").is_none(),
            )?
            .ok_or_else(|| external_error("default order quantity is unavailable"))?
        };
        if !quantity.is_finite() || quantity < 0.0 {
            return Err(external_error("quantity must be positive and finite"));
        }
        Ok(quantity)
    }
}
