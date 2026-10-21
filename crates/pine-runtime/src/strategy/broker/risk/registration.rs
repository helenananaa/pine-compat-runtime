use super::{RiskDrawdownType, RiskEntryDirection, RiskStopLimit};
use crate::{BrokerState, StrategyRiskConfiguration};

impl BrokerState {
    pub(crate) fn configure_native_risk(&mut self, configuration: &StrategyRiskConfiguration) {
        for rule in &configuration.rules {
            match rule.function.as_str() {
                "strategy.risk.allow_entry_in" => {
                    let next = RiskEntryDirection::from_strategy_constant(
                        rule.direction.as_deref().unwrap(),
                    )
                    .unwrap();
                    self.risk_rules.allow_entry_direction =
                        Some(match self.risk_rules.allow_entry_direction {
                            None | Some(RiskEntryDirection::All) => next,
                            Some(previous)
                                if next == RiskEntryDirection::All || previous == next =>
                            {
                                previous
                            }
                            Some(_) => RiskEntryDirection::Neither,
                        });
                }
                "strategy.risk.max_position_size" => {
                    let value = rule.value.unwrap();
                    if self
                        .risk_rules
                        .max_position_size
                        .is_none_or(|previous| value < previous)
                    {
                        self.set_max_position_size(value);
                    }
                }
                "strategy.risk.max_intraday_filled_orders" => {
                    let value = rule.value.unwrap();
                    if self
                        .risk_rules
                        .max_intraday_filled_orders
                        .is_none_or(|previous| value < f64::from(previous))
                    {
                        self.set_max_intraday_filled_orders(value, rule.alert_message.clone());
                    }
                }
                "strategy.risk.max_cons_loss_days" => {
                    let value = rule.value.unwrap();
                    if self
                        .risk_rules
                        .max_cons_loss_days
                        .is_none_or(|previous| value < f64::from(previous))
                    {
                        self.set_max_cons_loss_days(value, rule.alert_message.clone());
                    }
                }
                _ => {}
            }
        }
        // Cash and percentage thresholds cannot be ordered numerically. Keep
        // the strictest declaration for each unit and evaluate both at ticks.
        let drawdown = monetary_limits(configuration, "strategy.risk.max_drawdown");
        if let Some(first) = drawdown.first() {
            self.set_max_drawdown(
                first.value,
                type_name(first.value_type),
                first.alert_message.clone(),
            );
            self.risk_rules.drawdown_limits = drawdown;
        }
        let intraday = monetary_limits(configuration, "strategy.risk.max_intraday_loss");
        if let Some(first) = intraday.first() {
            self.set_max_intraday_loss(
                first.value,
                type_name(first.value_type),
                first.alert_message.clone(),
            );
            self.risk_rules.intraday_loss_limits = intraday;
        }
    }
}

fn type_name(kind: RiskDrawdownType) -> &'static str {
    match kind {
        RiskDrawdownType::Cash => "strategy.cash",
        RiskDrawdownType::PercentOfEquity => "strategy.percent_of_equity",
    }
}

fn monetary_limits(
    configuration: &StrategyRiskConfiguration,
    function: &str,
) -> Vec<RiskStopLimit> {
    let mut limits: Vec<RiskStopLimit> = Vec::new();
    for rule in configuration
        .rules
        .iter()
        .filter(|rule| rule.function == function)
    {
        let value_type =
            RiskDrawdownType::from_strategy_constant(rule.value_type.as_deref().unwrap()).unwrap();
        let candidate = RiskStopLimit {
            value: rule.value.unwrap(),
            value_type,
            alert_message: rule.alert_message.clone(),
        };
        if let Some(previous) = limits
            .iter_mut()
            .find(|limit| limit.value_type == value_type)
        {
            if candidate.value < previous.value {
                *previous = candidate;
            }
        } else {
            limits.push(candidate);
        }
    }
    limits
}
