//! Optional application policy for running alerts and external delivery.
//!
//! This crate depends on the interpreter's public events and neutral template
//! renderer. The interpreter has no dependency on these host helpers.
//!
//! ```
//! use pine_host_support::{RunningAlertConfig, strategy_order_fill_delivery_candidate};
//! use pine_runtime::StrategyOrderFillAlertOutput;
//!
//! let config = RunningAlertConfig::new_strategy_order_fills(
//!     "snapshot-1", "EXCHANGE:SYMBOL", "60", "Fill: {{strategy.order.alert_message}}",
//! );
//! let event = StrategyOrderFillAlertOutput {
//!     id: "entry-1".into(), bar_index: 3, time: 1_000,
//!     direction: "strategy.entry".into(), qty: 1.0, price: 10.0,
//!     entry_id: Some("entry-1".into()), exit_id: None, message: "buy".into(),
//! };
//! let candidate = strategy_order_fill_delivery_candidate("running-1", &config, &event)?;
//! assert_eq!(candidate.rendered_message, "Fill: buy");
//! # Ok::<(), pine_host_support::RunningAlertEvaluationError>(())
//! ```

pub mod delivery;
pub mod running_alerts;

pub use delivery::*;
pub use running_alerts::*;
