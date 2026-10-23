use pine_ir::{HirExpr, HirExprKind, HirLiteral, HirProgram, SymbolId};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRateInputContract {
    pub schema_version: u32,
    pub profile: &'static str,
    pub account_conversion: &'static str,
    pub historical_reference: &'static str,
    pub realtime_reference: &'static str,
    pub missing_pair: &'static str,
    pub before_first_rate: &'static str,
    pub requests: Vec<CurrencyRateRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRateRequirement {
    pub call_site_id: u32,
    pub from: CurrencyArgument,
    pub to: CurrencyArgument,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum CurrencyArgument {
    Literal(String),
    ChartQuoteCurrency,
    DeclaredAccountOrChartQuoteCurrency,
    RuntimeExpression,
}

pub(super) fn argument(
    _program: &HirProgram,
    names: &HashMap<SymbolId, &str>,
    expr: Option<&HirExpr>,
) -> CurrencyArgument {
    let Some(expr) = expr else {
        return CurrencyArgument::RuntimeExpression;
    };
    // Input-default evaluation can substitute default chart metadata (USD).
    // Inventory must retain host-dependent selectors rather than freezing it.
    if let HirExprKind::Literal(HirLiteral::String(value)) = &expr.kind {
        return CurrencyArgument::Literal(value.clone());
    }
    match super::builtin_name(names, expr) {
        Some("syminfo.currency") => CurrencyArgument::ChartQuoteCurrency,
        Some("strategy.account_currency") => CurrencyArgument::DeclaredAccountOrChartQuoteCurrency,
        _ => CurrencyArgument::RuntimeExpression,
    }
}
