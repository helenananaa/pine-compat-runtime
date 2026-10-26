use super::BrokerState;
use super::ledger::{OpenTrade, TradeDirection};
use crate::{PineValue, StrategyEquitySnapshot, StrategyTrade};

fn normalize_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

impl BrokerState {
    fn mark_price(&self, price: f64) -> f64 {
        let Some(tick) = self.price_tick else {
            return price;
        };
        if !price.is_finite() {
            return price;
        }
        let ticks = price / tick;
        if (ticks - ticks.round()).abs() <= 1e-8 {
            return price;
        }
        // Native half-tick marks round the quotient. Adding 0.5 first can
        // move a quotient just below the midpoint onto the opposite tick.
        ticks.round() * tick
    }

    pub(crate) fn record_equity(&mut self, bar_index: usize, close: f64) {
        let close = self.mark_price(close);
        let market_value = self.contract_value(self.position_size * close);
        let equity = self.cash + market_value;
        let net_profit = normalize_zero(equity - self.initial_capital);
        let snapshot = StrategyEquitySnapshot {
            bar_index,
            cash: self.cash,
            market_value,
            equity,
            net_profit,
        };
        if self
            .equity
            .last()
            .is_some_and(|last| last.bar_index == bar_index)
        {
            *self.equity.last_mut().expect("same-bar equity row exists") = snapshot;
        } else {
            self.equity.push(snapshot);
        }
    }

    pub(crate) fn update_open_trade_extremes(&mut self, high: f64, low: f64) {
        if self.open_trade_count() <= 0 {
            return;
        }
        self.trade_ledger
            .update_extremes(high, low, self.contract_point_value_to_account());
        if self.open_trade_count() == 1 {
            if high.is_finite() {
                self.open_trade_max_high = Some(
                    self.open_trade_max_high
                        .map_or(high, |value| value.max(high)),
                );
            }
            if low.is_finite() {
                self.open_trade_min_low =
                    Some(self.open_trade_min_low.map_or(low, |value| value.min(low)));
            }
        }
        self.observe_strategy_equity_extremes(high);
        self.observe_strategy_equity_extremes(low);
    }

    #[must_use]
    pub(crate) fn open_profit(&self, close: f64) -> f64 {
        let close = self.mark_price(close);
        if self.position_size != 0.0 {
            normalize_zero(self.contract_value((close - self.avg_price) * self.position_size))
        } else {
            0.0
        }
    }

    #[must_use]
    pub(crate) fn open_profit_percent(&self, close: f64) -> Option<f64> {
        let open_profit = self.open_profit(close);
        let realized_equity = self.initial_capital + self.net_profit();
        if !open_profit.is_finite() || !realized_equity.is_finite() || realized_equity <= 0.0 {
            return None;
        }
        Some(normalize_zero(open_profit / realized_equity * 100.0))
    }

    #[must_use]
    pub(crate) fn open_trade_capital_held(&self, close: f64) -> Option<f64> {
        let has_active_margin = self.margin_long.is_active() || self.margin_short.is_active();
        if !has_active_margin {
            return None;
        }
        if self.position_size == 0.0 {
            return Some(0.0);
        }
        if !close.is_finite() {
            return None;
        }
        if self.position_size > 0.0 {
            return self.margin_required_for_position(close);
        }
        self.margin_required_for_position(close).or(Some(0.0))
    }

    #[must_use]
    fn margin_ratio_for_direction(&self, direction: TradeDirection) -> Option<f64> {
        let setting = match direction {
            TradeDirection::Long => self.margin_long,
            TradeDirection::Short => self.margin_short,
        };
        if !setting.is_active() {
            return None;
        }
        let ratio = setting.value_percent / 100.0;
        ratio.is_finite().then_some(ratio)
    }

    #[must_use]
    pub(crate) fn margin_required_for_position(&self, price: f64) -> Option<f64> {
        if self.position_size == 0.0 {
            return if self.margin_long.is_active() || self.margin_short.is_active() {
                Some(0.0)
            } else {
                None
            };
        }
        if !price.is_finite() {
            return None;
        }
        let direction = if self.position_size > 0.0 {
            TradeDirection::Long
        } else {
            TradeDirection::Short
        };
        let ratio = self.margin_ratio_for_direction(direction)?;
        Some(normalize_zero(
            self.contract_value(self.position_size.abs() * price * ratio),
        ))
    }

    #[must_use]
    pub(crate) fn margin_liquidation_price(&self) -> Option<f64> {
        if self.position_size > 0.0 {
            if !self.margin_long.is_active() {
                return None;
            }
            let margin_ratio = self.margin_long.value_percent / 100.0;
            if !margin_ratio.is_finite() || margin_ratio <= 0.0 {
                return None;
            }
            let denominator = self.contract_value(self.position_size * (1.0 - margin_ratio));
            if !denominator.is_finite() || denominator == 0.0 {
                return None;
            }
            let price = -self.cash / denominator;
            return price.is_finite().then(|| normalize_zero(price));
        }
        if self.position_size >= 0.0 || !self.margin_short.is_active() {
            return None;
        }
        let margin_ratio = self.margin_short.value_percent / 100.0;
        if !margin_ratio.is_finite() || margin_ratio <= 0.0 {
            return None;
        }
        let denominator = self.contract_value(self.position_size.abs() * (1.0 + margin_ratio));
        if !denominator.is_finite() || denominator == 0.0 {
            return None;
        }
        let price = self.cash / denominator;
        if price.is_finite() {
            Some(normalize_zero(price))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn can_afford_long_entry(&self, qty: f64, fill_price: f64) -> bool {
        let Some(ratio) = self.margin_ratio_for_direction(TradeDirection::Long) else {
            return true;
        };
        let required_margin = self.contract_value(qty * fill_price * ratio);
        required_margin.is_finite() && self.equity_value(fill_price) >= required_margin
    }

    #[must_use]
    pub(crate) fn can_afford_short_entry(&self, qty: f64, fill_price: f64) -> bool {
        let Some(ratio) = self.margin_ratio_for_direction(TradeDirection::Short) else {
            return true;
        };
        let required_margin = self.contract_value(qty * fill_price * ratio);
        required_margin.is_finite() && self.equity_value(fill_price) >= required_margin
    }

    #[must_use]
    pub(crate) fn initial_capital(&self) -> f64 {
        self.initial_capital
    }

    #[must_use]
    pub(crate) fn realized_profit(&self) -> f64 {
        normalize_zero(self.realized_profit_sum)
    }

    #[must_use]
    pub(crate) fn net_profit(&self) -> f64 {
        // Closed trades include allocated fees; subtract only fees still held
        // by open exposure to account for entry commission exactly once.
        normalize_zero(
            self.realized_profit()
                - self
                    .trade_ledger
                    .open_trades()
                    .iter()
                    .map(|trade| trade.entry_commission)
                    .sum::<f64>(),
        )
    }

    #[must_use]
    pub(crate) fn realized_profit_percent(&self) -> f64 {
        self.initial_capital_percent(self.net_profit())
    }

    #[must_use]
    pub(crate) fn gross_profit(&self) -> f64 {
        normalize_zero(self.closed_trade_stats.gross_profit())
    }

    #[must_use]
    pub(crate) fn gross_profit_percent(&self) -> f64 {
        self.initial_capital_percent(self.gross_profit())
    }

    #[must_use]
    pub(crate) fn gross_loss(&self) -> f64 {
        normalize_zero(self.closed_trade_stats.gross_loss())
    }

    #[must_use]
    pub(crate) fn gross_loss_percent(&self) -> f64 {
        self.initial_capital_percent(self.gross_loss())
    }

    #[must_use]
    pub(crate) fn average_trade(&self) -> Option<f64> {
        if self.trades.is_empty() {
            None
        } else {
            Some(normalize_zero(
                self.realized_profit() / self.trades.len() as f64,
            ))
        }
    }

    #[must_use]
    pub(crate) fn average_winning_trade(&self) -> Option<f64> {
        self.closed_trade_stats
            .average_winning_trade()
            .map(normalize_zero)
    }

    #[must_use]
    pub(crate) fn average_losing_trade(&self) -> Option<f64> {
        self.closed_trade_stats
            .average_losing_trade()
            .map(normalize_zero)
    }

    #[must_use]
    pub(crate) fn average_trade_percent(&self) -> Option<f64> {
        self.closed_trade_stats
            .average_trade_percent()
            .map(normalize_zero)
    }

    #[must_use]
    pub(crate) fn average_winning_trade_percent(&self) -> Option<f64> {
        self.closed_trade_stats
            .average_winning_trade_percent()
            .map(normalize_zero)
    }

    #[must_use]
    pub(crate) fn average_losing_trade_percent(&self) -> Option<f64> {
        self.closed_trade_stats
            .average_losing_trade_percent()
            .map(normalize_zero)
    }

    #[must_use]
    pub(crate) fn max_drawdown(&self) -> f64 {
        normalize_zero(self.max_drawdown)
    }

    #[must_use]
    pub(crate) fn max_drawdown_percent(&self) -> f64 {
        normalize_zero(self.max_drawdown_percent)
    }

    #[must_use]
    pub(crate) fn max_runup(&self) -> f64 {
        normalize_zero(self.max_runup)
    }

    #[must_use]
    pub(crate) fn max_runup_percent(&self) -> f64 {
        normalize_zero(self.max_runup_percent)
    }

    #[must_use]
    pub(crate) fn max_contracts_held_all(&self) -> f64 {
        self.max_contracts_held_long()
            .max(self.max_contracts_held_short())
    }

    #[must_use]
    pub(crate) fn max_contracts_held_long(&self) -> f64 {
        normalize_zero(self.max_contracts_held_long)
    }

    #[must_use]
    pub(crate) fn max_contracts_held_short(&self) -> f64 {
        normalize_zero(self.max_contracts_held_short)
    }

    #[must_use]
    pub(crate) fn equity_value(&self, close: f64) -> f64 {
        let close = self.mark_price(close);
        normalize_zero(self.cash + self.contract_value(self.position_size * close))
    }

    fn initial_capital_percent(&self, value: f64) -> f64 {
        if !value.is_finite() || !self.initial_capital.is_finite() || self.initial_capital <= 0.0 {
            return 0.0;
        }
        normalize_zero(value / self.initial_capital * 100.0)
    }

    #[must_use]
    pub(crate) fn position_size(&self) -> f64 {
        self.position_size
    }

    #[must_use]
    pub fn closed_trade_count(&self) -> i64 {
        i64::try_from(self.trades.len()).unwrap_or(i64::MAX)
    }

    #[must_use]
    pub(crate) fn first_closed_trade_index(&self) -> i64 {
        // The local broker retains every closed trade, so the oldest retained
        // trade keeps its original zero-based index even before one exists.
        0
    }

    #[must_use]
    pub(crate) fn closed_trade(&self, trade_num: i64) -> Option<&StrategyTrade> {
        let index = usize::try_from(trade_num).ok()?;
        self.trades.get(index)
    }

    #[must_use]
    pub(crate) fn closed_trade_max_runup(&self, trade_num: i64) -> Option<f64> {
        let index = usize::try_from(trade_num).ok()?;
        self.closed_trade_metrics
            .get(index)
            .map(|metrics| metrics.max_runup)
    }

    #[must_use]
    pub(crate) fn closed_trade_profit_percent(&self, trade_num: i64) -> Option<f64> {
        let index = usize::try_from(trade_num).ok()?;
        self.closed_trade_metrics
            .get(index)
            .map(|metrics| metrics.profit_percent)
    }

    #[must_use]
    pub(crate) fn closed_trade_max_runup_percent(&self, trade_num: i64) -> Option<f64> {
        let index = usize::try_from(trade_num).ok()?;
        self.closed_trade_metrics
            .get(index)
            .map(|metrics| metrics.max_runup_percent)
    }

    #[must_use]
    pub(crate) fn closed_trade_commission(&self, trade_num: i64) -> Option<f64> {
        let index = usize::try_from(trade_num).ok()?;
        self.closed_trade_metrics
            .get(index)
            .map(|metrics| metrics.commission)
    }

    #[must_use]
    pub(crate) fn closed_trade_max_drawdown(&self, trade_num: i64) -> Option<f64> {
        let index = usize::try_from(trade_num).ok()?;
        self.closed_trade_metrics
            .get(index)
            .map(|metrics| metrics.max_drawdown)
    }

    #[must_use]
    pub(crate) fn closed_trade_max_drawdown_percent(&self, trade_num: i64) -> Option<f64> {
        let index = usize::try_from(trade_num).ok()?;
        self.closed_trade_metrics
            .get(index)
            .map(|metrics| metrics.max_drawdown_percent)
    }

    #[must_use]
    pub(crate) fn closed_trade_entry_comment(&self, trade_num: i64) -> Option<&str> {
        let index = usize::try_from(trade_num).ok()?;
        self.closed_trade_metrics
            .get(index)
            .and_then(|metrics| metrics.entry_comment.as_deref())
    }

    #[must_use]
    pub(crate) fn closed_trade_exit_comment(&self, trade_num: i64) -> Option<&str> {
        let index = usize::try_from(trade_num).ok()?;
        self.closed_trade_metrics
            .get(index)
            .and_then(|metrics| metrics.exit_comment.as_deref())
    }

    #[must_use]
    pub fn winning_trade_count(&self) -> i64 {
        i64::try_from(self.closed_trade_stats.winning_count()).unwrap_or(i64::MAX)
    }

    #[must_use]
    pub fn losing_trade_count(&self) -> i64 {
        i64::try_from(self.closed_trade_stats.losing_count()).unwrap_or(i64::MAX)
    }

    #[must_use]
    pub fn even_trade_count(&self) -> i64 {
        i64::try_from(self.closed_trade_stats.even_count()).unwrap_or(i64::MAX)
    }

    #[must_use]
    pub fn open_trade_count(&self) -> i64 {
        i64::try_from(self.trade_ledger.open_count()).unwrap_or(i64::MAX)
    }

    fn open_trade_at(&self, trade_num: i64) -> Option<&OpenTrade> {
        let index = usize::try_from(trade_num).ok()?;
        self.trade_ledger.open_at(index)
    }

    #[must_use]
    pub(crate) fn open_trade_entry_price(&self, trade_num: i64) -> Option<f64> {
        self.open_trade_at(trade_num).map(|trade| trade.entry_price)
    }

    #[must_use]
    pub(crate) fn open_trade_entry_id(&self, trade_num: i64) -> Option<&str> {
        self.open_trade_at(trade_num).map(|trade| trade.id.as_str())
    }

    #[must_use]
    pub(crate) fn open_trade_entry_comment(&self, trade_num: i64) -> Option<&str> {
        self.open_trade_at(trade_num)
            .and_then(|trade| trade.entry_metadata.comment.as_deref())
    }

    #[must_use]
    pub(crate) fn open_trade_entry_bar_index(&self, trade_num: i64) -> Option<usize> {
        self.open_trade_at(trade_num)
            .map(|trade| trade.entry_bar_index)
    }

    #[must_use]
    pub(crate) fn open_trade_entry_time(&self, trade_num: i64) -> Option<i64> {
        self.open_trade_at(trade_num).map(|trade| trade.entry_time)
    }

    #[must_use]
    pub(crate) fn open_trade_size(&self, trade_num: i64) -> Option<f64> {
        self.open_trade_at(trade_num)
            .map(|trade| trade.direction.signed_quantity(trade.quantity))
    }

    #[must_use]
    pub(crate) fn open_trade_profit(&self, trade_num: i64, close: f64) -> Option<f64> {
        let close = self.mark_price(close);
        self.open_trade_at(trade_num).map(|trade| {
            normalize_zero(
                self.contract_value(
                    (close - trade.entry_price) * trade.direction.signed_quantity(trade.quantity),
                ) - trade.entry_commission
                    - self.exit_commission_for_fill(trade.quantity, close),
            )
        })
    }

    #[must_use]
    pub(crate) fn open_trade_profit_percent(&self, trade_num: i64, close: f64) -> Option<f64> {
        let trade = self.open_trade_at(trade_num)?;
        let amount = self.open_trade_profit(trade_num, close)?;
        let capital =
            self.contract_value(trade.entry_price * trade.quantity) + trade.entry_commission;
        if !amount.is_finite() || !capital.is_finite() || capital <= 0.0 {
            return None;
        }
        Some(normalize_zero(amount / capital * 100.0))
    }

    #[must_use]
    pub(crate) fn open_trade_commission(&self, trade_num: i64) -> Option<f64> {
        self.open_trade_at(trade_num)
            .map(|trade| normalize_zero(trade.entry_commission))
    }

    #[must_use]
    pub(crate) fn open_trade_max_runup(&self, trade_num: i64) -> Option<f64> {
        self.open_trade_at(trade_num)
            .map(|trade| normalize_zero(trade.extrema.runup * trade.quantity))
    }

    #[must_use]
    pub(crate) fn open_trade_max_runup_percent(&self, trade_num: i64) -> Option<f64> {
        self.open_trade_at(trade_num)
            .map(|trade| trade.extrema.runup_percent)
    }

    #[must_use]
    pub(crate) fn open_trade_max_drawdown(&self, trade_num: i64) -> Option<f64> {
        self.open_trade_at(trade_num)
            .map(|trade| normalize_zero(trade.extrema.drawdown * trade.quantity))
    }

    #[must_use]
    pub(crate) fn open_trade_max_drawdown_percent(&self, trade_num: i64) -> Option<f64> {
        self.open_trade_at(trade_num)
            .map(|trade| trade.extrema.drawdown_percent)
    }

    #[must_use]
    pub(crate) fn position_avg_price_value(&self) -> PineValue {
        if self.position_size != 0.0 {
            PineValue::Float(self.avg_price)
        } else {
            PineValue::Na
        }
    }

    #[must_use]
    pub(crate) fn position_entry_name_value(&self) -> PineValue {
        if self.position_size == 0.0 {
            return PineValue::Na;
        }
        self.position_entry_name
            .as_ref()
            .map_or(PineValue::Na, |name| PineValue::String(name.clone()))
    }
}

impl super::BrokerState {
    pub(super) fn margin_call_quantity(&self, current_price: f64) -> Option<f64> {
        if !current_price.is_finite() || current_price <= 0.0 {
            return None;
        }
        if self.position_size > 0.0 && self.margin_long.is_active() {
            let margin_ratio = self.margin_long.value_percent / 100.0;
            if !margin_ratio.is_finite() || margin_ratio <= 0.0 {
                return None;
            }
            let margin_required =
                self.contract_value(self.position_size * current_price * margin_ratio);
            let available_funds = self.equity_value(current_price) - margin_required;
            if !available_funds.is_finite() || available_funds >= 0.0 {
                return None;
            }
            let cover_amount = self
                .quantity_rules
                .margin_cover(available_funds / margin_ratio / self.contract_value(current_price));
            let qty = (cover_amount * 4.0).abs().min(self.position_size);
            return (qty.is_finite() && qty > 0.0).then_some(qty);
        }
        if self.position_size < 0.0 && self.margin_short.is_active() {
            let margin_ratio = self.margin_short.value_percent / 100.0;
            if !margin_ratio.is_finite() || margin_ratio <= 0.0 {
                return None;
            }
            let margin_required = self.margin_required_for_position(current_price)?;
            let available_funds = self.equity_value(current_price) - margin_required;
            if !available_funds.is_finite() || available_funds >= 0.0 {
                return None;
            }
            let cover_amount = self
                .quantity_rules
                .margin_cover(available_funds / margin_ratio / self.contract_value(current_price));
            let qty = (cover_amount * 4.0).abs().min(self.position_size.abs());
            return (qty.is_finite() && qty > 0.0).then_some(qty);
        }
        None
    }
}
