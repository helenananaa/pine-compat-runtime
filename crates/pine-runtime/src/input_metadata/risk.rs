//! Declarative risk configuration discovered independently of branch execution.
use crate::{ChartContext, InputOverrides, PineValue};
use pine_ir::{HirExprKind, HirProgram};
use serde::Serialize;

pub const STRATEGY_RISK_CONFIGURATION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRiskConfiguration {
    pub schema_version: u32,
    pub rules: Vec<StrategyRiskRule>,
    pub unresolved_call_site_ids: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRiskRule {
    pub call_site_id: u32,
    pub function: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_message: Option<String>,
    pub resolved: bool,
}

/// Resolve immutable scalar settings without executing branches, loops, orders,
/// requests or user side effects. Every physical call remains in the result;
/// hosts must apply all rules, including multiple calls of the same kind.
#[must_use]
pub fn strategy_risk_configuration(
    program: &HirProgram,
    overrides: &InputOverrides,
    chart: &ChartContext,
) -> StrategyRiskConfiguration {
    let minimal = super::constant::minimal_program(program.language_version);
    let mut evaluator =
        super::constant::ConstEvaluator::for_risk(program, &minimal, overrides, chart);
    let mut rules = Vec::new();
    crate::runtime::hir_walk::statements(&program.statements, &mut |expr| {
        let HirExprKind::Call {
            callee,
            call_site_id,
            args,
        } = &expr.kind
        else {
            return;
        };
        if !callee.starts_with("strategy.risk.") {
            return;
        }
        let mut evaluate = |index, name| {
            super::input_arg(args, index, name).and_then(|value| evaluator.value(value))
        };
        let mut rule = StrategyRiskRule {
            call_site_id: call_site_id.0,
            function: callee.clone(),
            value: None,
            value_type: None,
            direction: None,
            alert_message: None,
            resolved: true,
        };
        if callee == "strategy.risk.allow_entry_in" {
            rule.direction = match evaluate(0, "value") {
                Some(PineValue::String(value))
                    if matches!(
                        value.as_str(),
                        "strategy.direction.all"
                            | "strategy.direction.long"
                            | "strategy.direction.short"
                    ) =>
                {
                    Some(value)
                }
                _ => None,
            };
            rule.resolved = rule.direction.is_some();
        } else {
            let name = match callee.as_str() {
                "strategy.risk.max_position_size" => "contracts",
                "strategy.risk.max_drawdown" | "strategy.risk.max_intraday_loss" => "value",
                "strategy.risk.max_intraday_filled_orders" | "strategy.risk.max_cons_loss_days" => {
                    "count"
                }
                _ => {
                    rule.resolved = false;
                    rules.push(rule);
                    return;
                }
            };
            rule.value = evaluate(0, name)
                .and_then(|value| value.as_f64())
                .filter(|value| value.is_finite() && *value > 0.0);
            let has_type = matches!(
                callee.as_str(),
                "strategy.risk.max_drawdown" | "strategy.risk.max_intraday_loss"
            );
            if has_type {
                rule.value_type = match evaluate(1, "type") {
                    Some(PineValue::String(value))
                        if matches!(
                            value.as_str(),
                            "strategy.cash" | "strategy.percent_of_equity"
                        ) =>
                    {
                        Some(value)
                    }
                    _ => None,
                };
                rule.resolved &= rule.value_type.is_some();
                if rule.value_type.as_deref() == Some("strategy.percent_of_equity") {
                    rule.resolved &= rule.value.is_some_and(|value| value <= 100.0);
                }
            }
            if name == "count" {
                rule.resolved &= rule
                    .value
                    .is_some_and(|value| value.fract() == 0.0 && value <= f64::from(u32::MAX));
            }
            rule.resolved &= rule.value.is_some();
            let alert_index = if has_type { 2 } else { 1 };
            if super::input_arg(args, alert_index, "alert_message").is_some() {
                rule.alert_message = match evaluate(alert_index, "alert_message") {
                    Some(PineValue::String(value)) => Some(value),
                    _ => None,
                };
                rule.resolved &= rule.alert_message.is_some();
            }
        }
        rules.push(rule);
    });
    let unresolved_call_site_ids = rules
        .iter()
        .filter(|rule| !rule.resolved)
        .map(|rule| rule.call_site_id)
        .collect();
    StrategyRiskConfiguration {
        schema_version: STRATEGY_RISK_CONFIGURATION_SCHEMA_VERSION,
        rules,
        unresolved_call_site_ids,
    }
}
