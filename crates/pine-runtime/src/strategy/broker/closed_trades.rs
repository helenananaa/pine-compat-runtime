use super::{
    BrokerState, ClosedTradeMetrics, StrategyOrderMetadata,
    ledger::{OpenTrade, TradeAllocation},
};
use crate::StrategyTrade;

fn normalize_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

fn closed_trade_profit_percent(
    entry_price: f64,
    qty: f64,
    entry_commission: f64,
    profit: f64,
    point_to_account_value: f64,
) -> f64 {
    let denominator = entry_price * qty.abs() * point_to_account_value + entry_commission;
    if !profit.is_finite() || !denominator.is_finite() || denominator <= 0.0 {
        return 0.0;
    }
    normalize_zero(profit / denominator * 100.0)
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct AllocatedEntryFill {
    pub(super) extrema: super::trade_extrema::TradeExtrema,
    pub(super) entry_price: f64,
    pub(super) entry_bar_index: usize,
    pub(super) entry_time: i64,
    pub(super) entry_commission: f64,
    pub(super) entry_metadata: StrategyOrderMetadata,
}

impl AllocatedEntryFill {
    pub(super) fn from_allocations(
        allocations: &[TradeAllocation],
        fallback_entry_price: f64,
        fallback_entry_bar_index: usize,
        fallback_entry_time: i64,
        fallback_entry_commission: f64,
    ) -> Self {
        let Some(first_allocation) = allocations.first() else {
            return Self {
                extrema: Default::default(),
                entry_price: fallback_entry_price,
                entry_bar_index: fallback_entry_bar_index,
                entry_time: fallback_entry_time,
                entry_commission: fallback_entry_commission,
                entry_metadata: StrategyOrderMetadata::default(),
            };
        };

        Self {
            extrema: first_allocation.extrema,
            entry_price: first_allocation.entry_price,
            entry_bar_index: first_allocation.entry_bar_index,
            entry_time: first_allocation.entry_time,
            entry_commission: allocations
                .iter()
                .map(|allocation| allocation.entry_commission)
                .sum(),
            entry_metadata: first_allocation.entry_metadata.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ClosedTradeFill {
    pub(super) entry_id: String,
    pub(super) exit_id: String,
    pub(super) entry_fill: AllocatedEntryFill,
    pub(super) exit_bar_index: usize,
    pub(super) exit_time: i64,
    pub(super) exit_price: f64,
    pub(super) qty: f64,
    pub(super) profit: f64,
    pub(super) commission: f64,
    pub(super) close_metadata: StrategyOrderMetadata,
}

impl BrokerState {
    pub(super) fn record_closed_trade_fill(&mut self, fill: ClosedTradeFill) {
        let mut extrema = fill.entry_fill.extrema;
        let quantity = fill.qty.abs();
        if quantity > 0.0 {
            // The exit fill itself is part of this trade's observed path,
            // including a gap, limit endpoint or adverse execution slippage.
            // Native trade extrema include entry fees but exclude exit fees.
            extrema.observe(
                fill.entry_fill.entry_price,
                fill.qty.signum(),
                fill.entry_fill.entry_commission / quantity,
                (fill.exit_price, fill.exit_price),
                self.contract_point_value_to_account(),
            );
        }
        let profit_percent = closed_trade_profit_percent(
            fill.entry_fill.entry_price,
            fill.qty,
            fill.entry_fill.entry_commission,
            fill.profit,
            self.contract_point_value_to_account(),
        );
        self.record_window_realized_pnl(fill.profit);
        // Preserve the trade-order left fold used by realized_profit(), once per close.
        self.realized_profit_sum += fill.profit;
        self.closed_trade_stats.record(fill.profit, profit_percent);
        self.trades.push(StrategyTrade {
            id: fill.entry_id,
            exit_id: fill.exit_id,
            entry_bar_index: fill.entry_fill.entry_bar_index,
            exit_bar_index: fill.exit_bar_index,
            entry_time: fill.entry_fill.entry_time,
            exit_time: fill.exit_time,
            entry_price: fill.entry_fill.entry_price,
            exit_price: fill.exit_price,
            qty: fill.qty,
            profit: fill.profit,
        });
        self.closed_trade_metrics.push(ClosedTradeMetrics {
            quote_to_account_rate: self.quote_to_account_rate,
            commission: fill.commission,
            profit_percent,
            max_runup: extrema.runup * quantity,
            max_runup_percent: extrema.runup_percent,
            max_drawdown: extrema.drawdown * quantity,
            max_drawdown_percent: extrema.drawdown_percent,
            entry_comment: fill.entry_fill.entry_metadata.comment.clone(),
            exit_comment: fill.close_metadata.comment.clone(),
            close_metadata: fill.close_metadata,
        });
    }

    pub(super) fn record_open_long_legacy_state(&mut self, trade: &OpenTrade) {
        if self.position_size == 0.0 {
            // Native run-up keeps the lowest realized balance reached when
            // opening a position, after its first entry fee. Pyramiding fees
            // debit equity without restarting that position's baseline.
            if let Some(equity) = trade.equity_on_entry {
                self.min_equity_before_open_trade = self
                    .min_equity_before_open_trade
                    .min(equity - trade.entry_commission);
            }
            self.position_entry_name = Some(trade.id.clone());
        }
        let signed_size = trade.direction.signed_quantity(trade.quantity);
        self.position_size = signed_size;
        if signed_size > 0.0 {
            self.max_contracts_held_long = self.max_contracts_held_long.max(signed_size);
        } else if signed_size < 0.0 {
            self.max_contracts_held_short = self.max_contracts_held_short.max(-signed_size);
        }
        self.avg_price = trade.entry_price;
        self.open_entry_commission = trade.entry_commission;
        self.entry_id = Some(trade.id.clone());
        self.entry_bar_index = Some(trade.entry_bar_index);
        self.entry_time = Some(trade.entry_time);
        self.open_trade_max_high = trade.max_high;
        self.open_trade_min_low = trade.min_low;
        self.open_trade_equity_on_entry = trade.equity_on_entry;
        self.open_trade_min_equity_before_entry = trade.min_equity_before_entry;
        self.open_trade_max_equity_before_entry = trade.max_equity_before_entry;
    }

    pub(super) fn clear_open_long_legacy_state(&mut self) {
        self.position_size = 0.0;
        self.avg_price = 0.0;
        self.entry_id = None;
        self.position_entry_name = None;
        self.entry_bar_index = None;
        self.entry_time = None;
        self.open_entry_commission = 0.0;
        self.open_trade_max_high = None;
        self.open_trade_min_low = None;
        self.open_trade_equity_on_entry = None;
        self.open_trade_min_equity_before_entry = None;
        self.open_trade_max_equity_before_entry = None;
    }
}
