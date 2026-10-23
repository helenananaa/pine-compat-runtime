//! Evaluated order annotations. Matching, fill selection and delivery are Host-owned.
use pine_ir::HirCallArg;
use serde::Serialize;

use crate::builtins::args::call_arg_expr;
use crate::strategy::StrategyOrderMetadata;
use crate::{HistoricalRuntime, RuntimeError};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ExternalOrderMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_profit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_loss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_trailing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_profit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_loss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_trailing: Option<String>,
    pub disable_alert: bool,
}

impl From<StrategyOrderMetadata> for ExternalOrderMetadata {
    fn from(value: StrategyOrderMetadata) -> Self {
        Self {
            comment: value.comment,
            alert_message: value.alert_message,
            disable_alert: value.disable_alert,
            ..Self::default()
        }
    }
}

impl HistoricalRuntime<'_> {
    pub(super) fn external_order_metadata(
        &mut self,
        callee: &str,
        args: &[HirCallArg],
    ) -> Result<Option<ExternalOrderMetadata>, RuntimeError> {
        if !self
            .external_execution
            .as_ref()
            .is_some_and(|state| state.options.order_metadata)
        {
            let first = match callee {
                "strategy.entry" | "strategy.order" => 7,
                "strategy.close" => 3,
                "strategy.close_all" => 0,
                "strategy.exit" => 12,
                _ => return Ok(None),
            };
            let names: &[&str] = if callee == "strategy.exit" {
                &[
                    "comment",
                    "comment_profit",
                    "comment_loss",
                    "comment_trailing",
                    "alert_message",
                    "alert_profit",
                    "alert_loss",
                    "alert_trailing",
                    "disable_alert",
                ]
            } else {
                &["comment", "alert_message", "disable_alert"]
            };
            if names
                .iter()
                .enumerate()
                .any(|(offset, name)| call_arg_expr(args, first + offset, name).is_some())
            {
                return Err(super::external_error(
                    "order metadata requires explicit Host opt-in",
                ));
            }
            return Ok(None);
        }
        let metadata = match callee {
            "strategy.entry" => self.eval_strategy_entry_metadata(args)?.into(),
            "strategy.order" => self.eval_strategy_order_metadata(args)?.into(),
            "strategy.close" => self.eval_strategy_close_metadata(args, 3)?.into(),
            "strategy.close_all" => self.eval_strategy_close_metadata(args, 0)?.into(),
            "strategy.exit" => {
                let value = self.eval_strategy_exit_metadata(args)?;
                ExternalOrderMetadata {
                    comment: value.comment,
                    comment_profit: value.comment_profit,
                    comment_loss: value.comment_loss,
                    comment_trailing: value.comment_trailing,
                    alert_message: value.alert_message,
                    alert_profit: value.alert_profit,
                    alert_loss: value.alert_loss,
                    alert_trailing: value.alert_trailing,
                    disable_alert: value.disable_alert,
                }
            }
            _ => return Ok(None),
        };
        Ok((metadata != ExternalOrderMetadata::default()).then_some(metadata))
    }
}
