//! Side-effect-free discovery of inputs used by an executable program.
//! This inventory is conservative: it does not prove branch reachability or
//! that a provider contains sufficient data for a particular execution.

use std::collections::{BTreeSet, HashMap};

use pine_ir::{HirExpr, HirExprKind, HirLiteral, HirProgram, ScriptMode, SymbolId};
use serde::Serialize;

mod walk;

pub const HOST_REQUIREMENTS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostRequirements {
    pub schema_version: u32,
    pub discovery: &'static str,
    pub chart: ChartInputContract,
    pub account: Option<AccountInputContract>,
    pub execution: ExecutionInputContract,
    pub requests: Vec<RequestRequirement>,
    pub input_call_site_ids: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartInputContract {
    pub bars: &'static str,
    pub price_grid: &'static str,
    pub quantity_precision: &'static str,
    pub symbol_metadata: Vec<String>,
    pub defaults: ChartInputDefaults,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartInputDefaults {
    pub symbol: String,
    pub timeframe: String,
    pub min_move: u32,
    pub price_scale: u32,
    pub quantity_precision: u32,
    pub synthetic: bool,
    pub symbol_lookup: bool,
    pub currency: &'static str,
    pub point_value: u32,
    pub timezone: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInputContract {
    pub profile: &'static str,
    pub point_value: u32,
    pub currency_conversion: bool,
    pub unsupported_profiles: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionInputContract {
    pub clock: &'static str,
    pub realtime_updates: &'static str,
    pub calc_on_every_tick: bool,
    pub calc_on_order_fills: bool,
    pub process_orders_on_close: bool,
    pub magnifier: &'static str,
    pub session_windows: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestRequirement {
    pub call_site_id: u32,
    pub function: String,
    pub symbol: RequestArgument,
    pub timeframe: RequestArgument,
    pub provider: &'static str,
    pub timeframe_relation: &'static str,
    pub gaps: &'static str,
    pub lookahead: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum RequestArgument {
    Literal(String),
    CurrentContextSymbol,
    CurrentContextTimeframe,
    RootChartSymbol,
    RuntimeExpression,
}

/// Inspect lowered executable calls, including inlined libraries, without
/// evaluating inputs, consulting data providers, or changing runtime state.
#[must_use]
pub fn host_requirements(program: &HirProgram) -> HostRequirements {
    let symbol_names: HashMap<SymbolId, &str> = program
        .symbols
        .iter()
        .map(|symbol| (symbol.id, symbol.name.as_str()))
        .collect();
    let defaults = crate::ChartContext::default();
    let mut requests = Vec::new();
    let mut metadata = BTreeSet::new();
    let mut input_ids = BTreeSet::new();
    let mut clock = false;
    let mut session_windows = false;
    walk::statements(&program.statements, &mut |expr| {
        if let Some(name) = builtin_name(&symbol_names, expr)
            && name.starts_with("syminfo.")
        {
            metadata.insert(name.to_owned());
        }
        clock |= matches!(&expr.kind, HirExprKind::Builtin(name) if name == "timenow")
            || matches!(&expr.kind, HirExprKind::Symbol(id) if Some(*id) == program.timenow_symbol);
        let HirExprKind::Call {
            callee,
            call_site_id,
            args,
        } = &expr.kind
        else {
            return;
        };
        if callee == "input" || callee.starts_with("input.") {
            input_ids.insert(call_site_id.0);
        }
        session_windows |= matches!(
            callee.as_str(),
            "strategy.risk.max_intraday_loss"
                | "strategy.risk.max_intraday_filled_orders"
                | "strategy.risk.max_cons_loss_days"
        );
        if callee == "request.security" || callee.starts_with("$legacy.security.") {
            let argument = |index, name, symbol| {
                crate::builtins::args::call_arg_expr(args, index, name)
                    .map_or(RequestArgument::RuntimeExpression, |value| {
                        request_argument(&symbol_names, value, symbol)
                    })
            };
            requests.push(RequestRequirement {
                call_site_id: call_site_id.0,
                function: if callee.starts_with("$legacy.security.") {
                    "security".to_owned()
                } else {
                    callee.clone()
                },
                symbol: argument(0, "symbol", true),
                timeframe: argument(1, "timeframe", false),
                provider: "whenEvaluatedOutsideCurrentContext",
                timeframe_relation: "sameOrHigherIntegerMultiple",
                gaps: if callee.contains(".gaps_on.") {
                    "gapsOn"
                } else {
                    "gapsOff"
                },
                lookahead: if callee.ends_with(".lookahead_on") {
                    "lookaheadOn"
                } else {
                    "lookaheadOff"
                },
            });
        }
    });
    requests.sort_by_key(|request| request.call_site_id);
    requests.dedup();
    let strategy = program.script_mode == ScriptMode::Strategy;
    HostRequirements {
        schema_version: HOST_REQUIREMENTS_SCHEMA_VERSION,
        discovery: "conservativeExecutableHirInventory",
        chart: ChartInputContract {
            bars: "hostSuppliedStandardOhlcv",
            price_grid: "positiveIntegerMinMoveAndPriceScale",
            quantity_precision: "decimalPowerZeroThroughNine",
            symbol_metadata: metadata.into_iter().collect(),
            defaults: ChartInputDefaults {
                symbol: defaults.symbol().to_owned(),
                timeframe: defaults.timeframe().value().to_owned(),
                min_move: defaults.min_move(),
                price_scale: defaults.price_scale(),
                quantity_precision: defaults.quantity_scale().ilog10(),
                synthetic: true,
                symbol_lookup: false,
                currency: "USD",
                point_value: 1,
                timezone: "Etc/UTC",
            },
        },
        account: strategy.then_some(AccountInputContract {
            profile: "linearUnitPointValueSameCurrency",
            point_value: 1,
            currency_conversion: false,
            unsupported_profiles: vec![
                "foreignCurrencyConversion",
                "nonUnitContractMultiplier",
                "nonstandardChartBroker",
            ],
        }),
        execution: ExecutionInputContract {
            clock: if clock {
                "explicitTimestampWhenEvaluated"
            } else {
                "notUsed"
            },
            realtime_updates: "hostOrderedFormingReplacementAndConfirmation",
            calc_on_every_tick: strategy && program.strategy_settings.calc_on_every_tick,
            calc_on_order_fills: strategy && program.strategy_settings.calc_on_order_fills,
            process_orders_on_close: strategy && program.strategy_settings.process_orders_on_close,
            magnifier: if strategy && program.strategy_settings.use_bar_magnifier {
                "historicalIntrabarsOrReportedStandardOhlcFallback"
            } else {
                "notEnabled"
            },
            session_windows: if session_windows {
                "hostWindowAndTradingDayIdsOrUtcFallback"
            } else {
                "notUsedByWindowRiskRules"
            },
        },
        requests,
        input_call_site_ids: input_ids.into_iter().collect(),
    }
}

#[must_use]
pub fn host_requirements_json(program: &HirProgram) -> String {
    serde_json::to_string(&host_requirements(program))
        .expect("host requirements contain only JSON-compatible strings, integers and booleans")
}

fn builtin_name<'a>(
    symbol_names: &HashMap<SymbolId, &'a str>,
    expr: &'a HirExpr,
) -> Option<&'a str> {
    match &expr.kind {
        HirExprKind::Builtin(name) => Some(name),
        HirExprKind::Symbol(id) => symbol_names.get(id).copied(),
        _ => None,
    }
}

fn request_argument(
    symbol_names: &HashMap<SymbolId, &str>,
    expr: &HirExpr,
    symbol: bool,
) -> RequestArgument {
    if let HirExprKind::Literal(HirLiteral::String(value)) = &expr.kind {
        return RequestArgument::Literal(value.clone());
    }
    match builtin_name(symbol_names, expr) {
        Some("syminfo.tickerid") if symbol => RequestArgument::CurrentContextSymbol,
        Some("syminfo.main_tickerid") if symbol => RequestArgument::RootChartSymbol,
        Some("timeframe.period") if !symbol => RequestArgument::CurrentContextTimeframe,
        _ => RequestArgument::RuntimeExpression,
    }
}
