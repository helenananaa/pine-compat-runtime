const STRATEGY_NEXT_TICK_CLOSE_BARS: &str =
    include_str!("../../../../tests/fixtures/runtime/strategy_next_tick_close_bars.csv");

fn strategy_next_tick_close_bars(fixture: &str) -> Option<&'static str> {
    matches!(
        fixture,
        "tests/fixtures/runtime/strategy_close.pine"
            | "tests/fixtures/runtime/strategy_close_all.pine"
            | "tests/fixtures/runtime/strategy_close_all_exit.pine"
            | "tests/fixtures/runtime/strategy_close_all_short.pine"
            | "tests/fixtures/runtime/strategy_close_short.pine"
            | "tests/fixtures/runtime/strategy_close_metadata.pine"
            | "tests/fixtures/runtime/strategy_close_noop.pine"
            | "tests/fixtures/runtime/strategy_close_qty_full_clamp.pine"
            | "tests/fixtures/runtime/strategy_close_qty_partial.pine"
            | "tests/fixtures/runtime/strategy_close_qty_percent_precedence.pine"
            | "tests/fixtures/runtime/strategy_close_entries_rule_any_close.pine"
            | "tests/fixtures/runtime/strategy_close_entries_rule_any_close_short.pine"
            | "tests/fixtures/runtime/strategy_close_entries_rule_fifo.pine"
            | "tests/fixtures/runtime/strategy_close_entries_rule_fifo_close_all.pine"
            | "tests/fixtures/runtime/strategy_closedtrades_fields.pine"
            | "tests/fixtures/runtime/strategy_closedtrades_fields_pyramiding.pine"
            | "tests/fixtures/runtime/strategy_pyramiding_close.pine"
            | "tests/fixtures/runtime/strategy_pyramiding_close_all.pine"
            | "tests/fixtures/runtime/strategy_equity.pine"
            | "tests/fixtures/runtime/strategy_position_state.pine"
            | "tests/fixtures/runtime/strategy_profit_state.pine"
            | "tests/fixtures/runtime/strategy_entry_metadata.pine"
            | "tests/fixtures/runtime/strategy_margin_capital_held_long.pine"
            | "tests/fixtures/runtime/strategy_margin_capital_held_short.pine"
            | "tests/fixtures/runtime/strategy_opentrades_fields.pine"
            | "tests/fixtures/runtime/strategy_variable_interactions.pine"
            | "tests/fixtures/runtime/strategy_exit_qty_percent_state.pine"
            | "tests/fixtures/runtime/strategy_exit_trailing_close_cancel.pine"
            | "tests/fixtures/runtime/strategy_commission_cash_per_contract.pine"
            | "tests/fixtures/runtime/strategy_commission_cash_per_order.pine"
            | "tests/fixtures/runtime/strategy_commission_percent.pine"
            | "tests/fixtures/runtime/strategy_commission_value_default_percent.pine"
            | "tests/fixtures/runtime/strategy_slippage.pine"
            | "tests/fixtures/runtime/strategy_trade_counts.pine"
            | "tests/fixtures/runtime/strategy_exit_qty_state.pine"
            | "tests/fixtures/runtime/strategy_profit_percent_state.pine"
    )
    .then_some(STRATEGY_NEXT_TICK_CLOSE_BARS)
}

pub(crate) fn runtime_fixture_bars_csv(fixture: &str) -> Option<&'static str> {
    if let Some(bars) = strategy_next_tick_close_bars(fixture) {
        return Some(bars);
    }
    match fixture {
        "tests/fixtures/runtime/strategy_risk_max_drawdown_cash.pine"
        | "tests/fixtures/runtime/strategy_risk_max_drawdown_percent.pine"
        | "tests/fixtures/runtime/strategy_risk_max_drawdown_blocks_order.pine"
        | "tests/fixtures/runtime/strategy_risk_max_intraday_loss_cash.pine"
        | "tests/fixtures/runtime/strategy_risk_max_intraday_loss_percent.pine" => Some(
            include_str!("../../../../tests/fixtures/runtime/strategy_risk_max_drawdown_bars.csv"),
        ),
        "tests/fixtures/runtime/strategy_risk_max_cons_loss_days.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/strategy_risk_max_cons_loss_days_bars.csv"
        )),
        "tests/fixtures/runtime/strategy_risk_max_cons_loss_days_no_trade.pine" => {
            Some(include_str!(
                "../../../../tests/fixtures/runtime/strategy_risk_max_cons_loss_days_no_trade_bars.csv"
            ))
        }
        "tests/fixtures/runtime/strategy_risk_max_intraday_filled_orders_reset.pine" => {
            Some(include_str!(
                "../../../../tests/fixtures/runtime/strategy_risk_max_intraday_filled_orders_reset_bars.csv"
            ))
        }
        "tests/fixtures/legacy/v2/runtime/core_legacy.pine" => Some(include_str!(
            "../../../../tests/fixtures/legacy/v2/runtime/core_bars.csv"
        )),
        "tests/fixtures/legacy/v3/runtime/core_legacy.pine" => Some(include_str!(
            "../../../../tests/fixtures/legacy/v3/runtime/core_bars.csv"
        )),
        "tests/fixtures/legacy/v4/runtime/logical_strict_legacy.pine" => Some(include_str!(
            "../../../../tests/fixtures/legacy/v4/runtime/logical_strict_bars.csv"
        )),
        "tests/fixtures/legacy/v4/runtime/session_defaults_legacy.pine" => Some(include_str!(
            "../../../../tests/fixtures/legacy/v4/runtime/session_weekend_bars.csv"
        )),
        "tests/fixtures/runtime/vwma_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/vwma_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/mfi_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/mfi_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/accdist_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/accdist_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/ao_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/ao_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/bop_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/bop_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/iii_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/iii_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/nvi_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/nvi_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/obv_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/obv_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/pvi_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/pvi_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/pvt_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/pvt_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/vwap_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/vwap_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/wad_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/wad_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/wvad_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/wvad_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/wpr_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/wpr_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/tr_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/tr_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/atr_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/atr_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/supertrend_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/supertrend_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/dmi_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/dmi_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/sar_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/sar_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/trend_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/trend_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/barssince_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/barssince_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/valuewhen_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/valuewhen_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/ema_rma_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/ema_rma_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/rsi_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/rsi_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/dema_tema_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/dema_tema_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/macd_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/macd_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/tsi_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/tsi_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/cmo_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/cmo_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/cci_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/cci_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/cog_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/cog_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/correlation_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/correlation_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/covariance_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/covariance_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/extremes_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/extremes_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/extreme_bars_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/extreme_bars_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/median_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/median_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/mode_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/mode_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/mom_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/mom_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/roc_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/roc_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/stoch_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/stoch_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/percentile_linear_interpolation_edge_cases.pine" => {
            Some(include_str!(
                "../../../../tests/fixtures/runtime/percentile_linear_interpolation_edge_cases_bars.csv"
            ))
        }
        "tests/fixtures/runtime/percentile_nearest_rank_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/percentile_nearest_rank_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/percentrank_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/percentrank_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/strategy_entry_stop_short.pine"
        | "tests/fixtures/runtime/strategy_order_stop_short.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/strategy_entry_stop_short_bars.csv"
        )),
        "tests/fixtures/runtime/strategy_entry_stop_limit_short.pine"
        | "tests/fixtures/runtime/strategy_order_stop_limit_short.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/strategy_entry_stop_limit_short_bars.csv"
        )),
        "tests/fixtures/runtime/strategy_margin_call_short.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/strategy_margin_call_short_bars.csv"
        )),
        "tests/fixtures/runtime/strategy_close_entries_rule_any_exit_from_entry_short.pine"
        | "tests/fixtures/runtime/strategy_close_entries_rule_any_exit_same_id_partial_short.pine" => {
            Some(include_str!(
                "../../../../tests/fixtures/runtime/strategy_close_entries_rule_any_exit_from_entry_short_bars.csv"
            ))
        }
        "tests/fixtures/runtime/strategy_ordinary_chart_up_gap_stop.pine"
        | "tests/fixtures/runtime/strategy_ordinary_chart_gap_stop_limit.pine" => {
            Some(include_str!(
                "../../../../tests/fixtures/runtime/strategy_ordinary_chart_up_gap_bars.csv"
            ))
        }
        "tests/fixtures/runtime/strategy_ordinary_chart_down_gap_limit.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/strategy_ordinary_chart_down_gap_bars.csv"
        )),
        "tests/fixtures/runtime/strategy_ordinary_chart_no_gap_stop.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/strategy_ordinary_chart_no_gap_bars.csv"
        )),
        _ => None,
    }
}
