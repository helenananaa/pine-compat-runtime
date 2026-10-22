mod broker;

use pine_ir::StrategyCommission;

pub(crate) fn percent_of_equity_order_qty(
    equity: f64,
    percent: f64,
    price: f64,
    commission: Option<StrategyCommission>,
    quantity_rules: crate::QuantityRules,
) -> f64 {
    let budget = equity * percent / 100.0;
    // Native percent sizing rounds the cash budget to ten significant decimal
    // digits before dividing by the execution price. This can move an order
    // across a contract boundary in either direction.
    let budget = if budget.is_finite() {
        format!("{budget:.9e}")
            .parse::<f64>()
            .expect("formatted finite budget")
    } else {
        budget
    };
    let qty = match commission {
        Some(StrategyCommission::Percent(rate)) => budget / (price * (1.0 + rate / 100.0)),
        Some(StrategyCommission::CashPerContract(fee)) => budget / (price + fee),
        Some(StrategyCommission::CashPerOrder(fee)) => (budget - fee).max(0.0) / price,
        None => budget / price,
    };
    quantity_on_chart_grid(qty, quantity_rules)
}

pub(crate) fn quantity_on_chart_grid(qty: f64, rules: crate::QuantityRules) -> f64 {
    rules.default_quantity(qty)
}

pub(crate) fn explicit_quantity_on_chart_grid(qty: f64, rules: crate::QuantityRules) -> f64 {
    rules.explicit_quantity(qty)
}

pub use broker::BrokerState;
pub(crate) use broker::PendingCloseQuantity;
pub(crate) use broker::{CombinedExitSpec, ExitQuantityRequest};
pub(crate) use broker::{EntryPathTick, PathEventOutcome};
pub(crate) use broker::{
    LossLimitBracketSpec, LossProfitBracketSpec, StopProfitBracketSpec, StrategyExitMetadata,
    StrategyOrderMetadata, TrailPointsExitSpec, TrailPriceExitSpec,
};
