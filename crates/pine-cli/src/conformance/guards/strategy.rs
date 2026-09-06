const STRATEGY_OCA_UNSUPPORTED_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/sema/unsupported_strategy_orders.pine",
    "tests/fixtures/sema/unsupported_strategy_exit_oca_name_series.pine",
    "tests/fixtures/sema/unsupported_strategy_order_oca_series_name.pine",
    "tests/fixtures/sema/unsupported_strategy_entry_oca_series_name.pine",
];

const STRATEGY_ORDER_OCA_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_order_oca_none.pine",
    "tests/fixtures/runtime/strategy_order_oca_cancel.pine",
    "tests/fixtures/runtime/strategy_order_oca_reduce.pine",
    "tests/fixtures/runtime/strategy_order_oca_reduce_zero.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_entry_order_cancel.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_entry_order_reduce.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_order_exit_reduce.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_none.pine",
    "tests/fixtures/sema/supported_strategy_order_oca_none.pine",
    "tests/fixtures/sema/supported_strategy_order_oca_cancel.pine",
    "tests/fixtures/sema/supported_strategy_order_oca_reduce.pine",
    "tests/fixtures/sema/unsupported_strategy_orders.pine",
    "tests/fixtures/sema/unsupported_strategy_order_oca_series_name.pine",
];

const STRATEGY_ENTRY_OCA_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_mixed_oca_entry_order_cancel.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_entry_order_reduce.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_none.pine",
    "tests/fixtures/sema/supported_strategy_entry_oca_none.pine",
    "tests/fixtures/sema/supported_strategy_entry_oca_cancel.pine",
    "tests/fixtures/sema/supported_strategy_entry_oca_reduce.pine",
    "tests/fixtures/sema/unsupported_strategy_entry_oca_series_name.pine",
];

const STRATEGY_EXECUTION_TIMING_BOUNDARY_FIXTURES: &[&str] =
    &["tests/fixtures/sema/unsupported_strategy_declaration_properties.pine"];

const STRATEGY_PROCESS_ORDERS_ON_CLOSE_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/sema/supported_strategy_process_orders_on_close.pine",
    "tests/fixtures/sema/unsupported_strategy_process_orders_on_close_series.pine",
    "tests/fixtures/sema/unsupported_strategy_process_orders_on_close_with_recalc.pine",
    "tests/fixtures/runtime/strategy_process_orders_on_close.pine",
    "tests/fixtures/runtime/strategy_process_orders_on_close_close.pine",
    "tests/fixtures/runtime/strategy_process_orders_on_close_immediately.pine",
];

const STRATEGY_CALC_ON_ORDER_FILLS_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/sema/supported_strategy_calc_on_order_fills.pine",
    "tests/fixtures/sema/unsupported_strategy_calc_on_order_fills_series.pine",
    "tests/fixtures/runtime/strategy_calc_on_order_fills.pine",
    "tests/fixtures/runtime/strategy_calc_on_order_fills_false.pine",
    "tests/fixtures/runtime/strategy_calc_on_order_fills_exit_avg.pine",
];

const STRATEGY_CALC_ON_EVERY_TICK_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/sema/supported_strategy_calc_on_every_tick.pine",
    "tests/fixtures/sema/unsupported_strategy_calc_on_every_tick_series.pine",
    "tests/fixtures/runtime/strategy_calc_on_every_tick.pine",
];

const STRATEGY_BAR_MAGNIFIER_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/sema/supported_strategy_use_bar_magnifier.pine",
    "tests/fixtures/sema/supported_strategy_use_bar_magnifier_v6.pine",
    "tests/fixtures/sema/supported_strategy_use_bar_magnifier_false.pine",
    "tests/fixtures/sema/unsupported_strategy_use_bar_magnifier_series.pine",
    "tests/fixtures/sema/unsupported_strategy_use_bar_magnifier_positional.pine",
    "tests/fixtures/runtime/strategy_use_bar_magnifier_fallback.pine",
    "tests/fixtures/runtime/strategy_use_bar_magnifier_false.pine",
    "tests/fixtures/runtime/strategy_use_bar_magnifier_gap.pine",
];

const STRATEGY_RISK_BOUNDARY_FIXTURES: &[&str] =
    &["tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine"];

const STRATEGY_RISK_ALLOW_ENTRY_IN_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_risk_allow_entry_in_long.pine",
    "tests/fixtures/runtime/strategy_risk_allow_entry_in_short.pine",
    "tests/fixtures/runtime/strategy_risk_allow_entry_in_long_flat_noop.pine",
    "tests/fixtures/runtime/strategy_risk_allow_entry_in_order_unaffected.pine",
    "tests/fixtures/runtime/strategy_risk_allow_entry_in_repeated.pine",
    "tests/fixtures/sema/supported_strategy_risk_allow_entry_in.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_allow_entry_in_unknown.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_allow_entry_in_series.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_allow_entry_in_indicator.pine",
    "tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine",
];

const STRATEGY_RISK_MAX_POSITION_SIZE_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_risk_max_position_size_reduces.pine",
    "tests/fixtures/runtime/strategy_risk_max_position_size_full_noop.pine",
    "tests/fixtures/runtime/strategy_risk_max_position_size_reversal.pine",
    "tests/fixtures/runtime/strategy_risk_max_position_size_order_unaffected.pine",
    "tests/fixtures/runtime/strategy_risk_max_position_size_pyramiding.pine",
    "tests/fixtures/runtime/strategy_risk_max_position_size_limit.pine",
    "tests/fixtures/sema/supported_strategy_risk_max_position_size.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_position_size_zero.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_position_size_negative.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_position_size_series.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_position_size_indicator.pine",
    "tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine",
];

const STRATEGY_RISK_MAX_DRAWDOWN_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_risk_max_drawdown_cash.pine",
    "tests/fixtures/runtime/strategy_risk_max_drawdown_percent.pine",
    "tests/fixtures/runtime/strategy_risk_max_drawdown_blocks_order.pine",
    "tests/fixtures/sema/supported_strategy_risk_max_drawdown.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_drawdown_zero.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_drawdown_percent_over.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_drawdown_unknown_type.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_drawdown_series.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_drawdown_indicator.pine",
    "tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine",
];

const STRATEGY_RISK_MAX_INTRADAY_LOSS_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_risk_max_intraday_loss_cash.pine",
    "tests/fixtures/runtime/strategy_risk_max_intraday_loss_percent.pine",
    "tests/fixtures/sema/supported_strategy_risk_max_intraday_loss.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_intraday_loss_zero.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_intraday_loss_percent_over.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_intraday_loss_unknown_type.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_intraday_loss_series.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_intraday_loss_indicator.pine",
    "tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine",
];

const STRATEGY_RISK_MAX_INTRADAY_FILLED_ORDERS_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_risk_max_intraday_filled_orders.pine",
    "tests/fixtures/runtime/strategy_risk_max_intraday_filled_orders_reset.pine",
    "tests/fixtures/sema/supported_strategy_risk_max_intraday_filled_orders.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_intraday_filled_orders_zero.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_intraday_filled_orders_fraction.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_intraday_filled_orders_series.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_intraday_filled_orders_indicator.pine",
    "tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine",
];

const STRATEGY_RISK_MAX_CONS_LOSS_DAYS_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_risk_max_cons_loss_days.pine",
    "tests/fixtures/runtime/strategy_risk_max_cons_loss_days_no_trade.pine",
    "tests/fixtures/sema/supported_strategy_risk_max_cons_loss_days.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_cons_loss_days_zero.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_cons_loss_days_fraction.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_cons_loss_days_series.pine",
    "tests/fixtures/sema/unsupported_strategy_risk_max_cons_loss_days_indicator.pine",
    "tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine",
];

const STRATEGY_ENTRY_SHORT_POSITIVE_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_entry_short.pine",
    "tests/fixtures/runtime/strategy_entry_short_reverses_long.pine",
    "tests/fixtures/runtime/strategy_entry_long_reverses_short.pine",
    "tests/fixtures/runtime/strategy_entry_limit_short.pine",
    "tests/fixtures/runtime/strategy_entry_stop_short.pine",
    "tests/fixtures/runtime/strategy_entry_stop_limit_short.pine",
    "tests/fixtures/runtime/strategy_entry_limit_reverses_short.pine",
    "tests/fixtures/runtime/strategy_entry_limit_reverses_long.pine",
    "tests/fixtures/runtime/strategy_entry_limit_reverses_short_qty.pine",
    "tests/fixtures/runtime/strategy_entry_stop_reverses_short.pine",
    "tests/fixtures/runtime/strategy_entry_stop_reverses_long.pine",
    "tests/fixtures/runtime/strategy_entry_stop_limit_reverses_short.pine",
    "tests/fixtures/runtime/strategy_entry_stop_limit_reverses_long.pine",
    "tests/fixtures/sema/supported_strategy_entry_short.pine",
    "tests/fixtures/sema/supported_strategy_entry_named_const_short_direction.pine",
    "tests/fixtures/sema/supported_strategy_entry_limit_short.pine",
    "tests/fixtures/sema/supported_strategy_entry_stop_short.pine",
    "tests/fixtures/sema/supported_strategy_entry_stop_limit_short.pine",
];

const STRATEGY_EXIT_SHORT_POSITIVE_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_exit_stop_short.pine",
    "tests/fixtures/runtime/strategy_exit_limit_short.pine",
    "tests/fixtures/runtime/strategy_exit_profit_short.pine",
    "tests/fixtures/runtime/strategy_exit_loss_short.pine",
    "tests/fixtures/runtime/strategy_exit_bracket_stop_limit_stop_fill_short.pine",
    "tests/fixtures/runtime/strategy_exit_bracket_stop_limit_limit_fill_short.pine",
    "tests/fixtures/runtime/strategy_exit_trail_price_fill_short.pine",
    "tests/fixtures/runtime/strategy_exit_trail_points_fill_short.pine",
];

const STRATEGY_MAX_CONTRACTS_HELD_SHORT_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_position_state.pine",
    "tests/fixtures/runtime/strategy_entry_short.pine",
];

const STRATEGY_ORDER_SHORT_REVERSAL_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_order_reduce_long.pine",
    "tests/fixtures/runtime/strategy_order_short_flat_noop.pine",
    "tests/fixtures/runtime/strategy_order_market_short_increase.pine",
    "tests/fixtures/runtime/strategy_order_long_flatten_short.pine",
    "tests/fixtures/runtime/strategy_order_long_reduce_short.pine",
    "tests/fixtures/runtime/strategy_order_short_flatten_long.pine",
    "tests/fixtures/runtime/strategy_order_long_against_short.pine",
    "tests/fixtures/runtime/strategy_order_short_oversized_against_long.pine",
    "tests/fixtures/runtime/strategy_order_limit_short.pine",
    "tests/fixtures/runtime/strategy_order_limit_long_against_short.pine",
    "tests/fixtures/runtime/strategy_order_limit_short_against_long.pine",
    "tests/fixtures/runtime/strategy_order_limit_long_flatten_short.pine",
    "tests/fixtures/runtime/strategy_order_limit_short_flatten_long.pine",
    "tests/fixtures/runtime/strategy_order_limit_long_reduce_short.pine",
    "tests/fixtures/runtime/strategy_order_limit_short_reduce_long.pine",
    "tests/fixtures/runtime/strategy_order_stop_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_long_against_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_short_against_long.pine",
    "tests/fixtures/runtime/strategy_order_stop_long_flatten_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_short_reduce_long.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_long_against_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_short_against_long.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_long_flatten_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_short_reduce_long.pine",
    "tests/fixtures/runtime/strategy_order_replace_limit_with_stop.pine",
    "tests/fixtures/runtime/strategy_order_replace_long_with_short.pine",
    "tests/fixtures/runtime/strategy_order_cancel_shared_id.pine",
    "tests/fixtures/runtime/strategy_order_reduce_fifo.pine",
    "tests/fixtures/runtime/strategy_order_reduce_any_matching_id.pine",
    "tests/fixtures/runtime/strategy_order_oca_none.pine",
    "tests/fixtures/runtime/strategy_order_oca_cancel.pine",
    "tests/fixtures/runtime/strategy_order_oca_reduce.pine",
    "tests/fixtures/runtime/strategy_order_oca_reduce_zero.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_entry_order_cancel.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_entry_order_reduce.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_order_exit_reduce.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_none.pine",
    "tests/fixtures/sema/unsupported_strategy_orders.pine",
];

const STRATEGY_ORDER_SUPPORTED_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_order_market_long.pine",
    "tests/fixtures/runtime/strategy_order_limit_long.pine",
    "tests/fixtures/runtime/strategy_order_limit_short.pine",
    "tests/fixtures/runtime/strategy_order_limit_long_against_short.pine",
    "tests/fixtures/runtime/strategy_order_limit_short_against_long.pine",
    "tests/fixtures/runtime/strategy_order_limit_long_flatten_short.pine",
    "tests/fixtures/runtime/strategy_order_limit_short_flatten_long.pine",
    "tests/fixtures/runtime/strategy_order_limit_long_reduce_short.pine",
    "tests/fixtures/runtime/strategy_order_limit_short_reduce_long.pine",
    "tests/fixtures/runtime/strategy_order_stop_long.pine",
    "tests/fixtures/runtime/strategy_order_stop_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_long.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_long_against_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_short_against_long.pine",
    "tests/fixtures/runtime/strategy_order_stop_long_flatten_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_short_reduce_long.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_long_against_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_short_against_long.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_long_flatten_short.pine",
    "tests/fixtures/runtime/strategy_order_stop_limit_short_reduce_long.pine",
    "tests/fixtures/runtime/strategy_order_reduce_long.pine",
    "tests/fixtures/runtime/strategy_order_short_flat_noop.pine",
    "tests/fixtures/runtime/strategy_order_market_short_increase.pine",
    "tests/fixtures/runtime/strategy_order_long_flatten_short.pine",
    "tests/fixtures/runtime/strategy_order_long_reduce_short.pine",
    "tests/fixtures/runtime/strategy_order_short_flatten_long.pine",
    "tests/fixtures/runtime/strategy_order_long_against_short.pine",
    "tests/fixtures/runtime/strategy_order_short_oversized_against_long.pine",
    "tests/fixtures/runtime/strategy_order_replace_limit_with_stop.pine",
    "tests/fixtures/runtime/strategy_order_replace_long_with_short.pine",
    "tests/fixtures/runtime/strategy_order_cancel_shared_id.pine",
    "tests/fixtures/runtime/strategy_order_reduce_fifo.pine",
    "tests/fixtures/runtime/strategy_order_reduce_any_matching_id.pine",
    "tests/fixtures/runtime/strategy_order_oca_none.pine",
    "tests/fixtures/runtime/strategy_order_oca_cancel.pine",
    "tests/fixtures/runtime/strategy_order_oca_reduce.pine",
    "tests/fixtures/runtime/strategy_order_oca_reduce_zero.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_entry_order_cancel.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_entry_order_reduce.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_order_exit_reduce.pine",
    "tests/fixtures/runtime/strategy_mixed_oca_none.pine",
    "tests/fixtures/sema/supported_strategy_order_oca_none.pine",
    "tests/fixtures/sema/supported_strategy_order_oca_cancel.pine",
    "tests/fixtures/sema/supported_strategy_order_oca_reduce.pine",
    "tests/fixtures/sema/unsupported_strategy_order_oca_series_name.pine",
    "tests/fixtures/runtime/strategy_order_metadata.pine",
    "tests/fixtures/sema/supported_strategy_order.pine",
    "tests/fixtures/sema/supported_strategy_order_metadata.pine",
    "tests/fixtures/sema/unsupported_strategy_orders.pine",
    "tests/fixtures/sema/unsupported_strategy_order_metadata_types.pine",
];

const STRATEGY_CANCEL_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_cancel_entry.pine",
    "tests/fixtures/runtime/strategy_cancel_exit.pine",
    "tests/fixtures/runtime/strategy_cancel_noop.pine",
    "tests/fixtures/runtime/strategy_order_cancel_shared_id.pine",
    "tests/fixtures/runtime/strategy_cancel_shared_id_entry_exit.pine",
    "tests/fixtures/runtime/strategy_cancel_shared_id_close_exit.pine",
    "tests/fixtures/sema/supported_strategy_cancel.pine",
];

const STRATEGY_CANCEL_ALL_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_cancel_all_entry_exit.pine",
    "tests/fixtures/runtime/strategy_cancel_all_exit.pine",
    "tests/fixtures/runtime/strategy_cancel_all_noop.pine",
    "tests/fixtures/runtime/strategy_cancel_all_families.pine",
    "tests/fixtures/sema/supported_strategy_cancel_all.pine",
];

const STRATEGY_CLOSE_ALL_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_close_all.pine",
    "tests/fixtures/runtime/strategy_close_all_short.pine",
    "tests/fixtures/runtime/strategy_close_all_exit.pine",
    "tests/fixtures/runtime/strategy_close_metadata.pine",
    "tests/fixtures/sema/supported_strategy_close_all.pine",
    "tests/fixtures/sema/supported_strategy_close_immediately.pine",
    "tests/fixtures/sema/supported_strategy_order_metadata.pine",
    "tests/fixtures/runtime/strategy_close_all_immediately.pine",
    "tests/fixtures/sema/unsupported_strategy_close_immediately.pine",
    "tests/fixtures/sema/unsupported_strategy_close_immediately_type.pine",
    "tests/fixtures/sema/unsupported_strategy_close_all_indicator.pine",
];

const STRATEGY_CLOSE_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_close.pine",
    "tests/fixtures/runtime/strategy_close_short.pine",
    "tests/fixtures/runtime/strategy_close_noop.pine",
    "tests/fixtures/runtime/strategy_close_exit.pine",
    "tests/fixtures/runtime/strategy_close_qty_partial.pine",
    "tests/fixtures/runtime/strategy_close_qty_full_clamp.pine",
    "tests/fixtures/runtime/strategy_close_qty_percent_precedence.pine",
    "tests/fixtures/sema/supported_strategy_close.pine",
    "tests/fixtures/sema/supported_strategy_close_qty.pine",
    "tests/fixtures/sema/supported_strategy_close_qty_percent.pine",
    "tests/fixtures/sema/supported_strategy_close_immediately.pine",
    "tests/fixtures/runtime/strategy_close_immediately.pine",
    "tests/fixtures/runtime/strategy_close_immediately_false.pine",
    "tests/fixtures/runtime/strategy_close_immediately_qty.pine",
    "tests/fixtures/runtime/strategy_close_immediately_short.pine",
    "tests/fixtures/sema/unsupported_strategy_close_immediately.pine",
    "tests/fixtures/sema/unsupported_strategy_close_immediately_type.pine",
    "tests/fixtures/sema/unsupported_strategy_close_indicator.pine",
];

const STRATEGY_MARGIN_ACCOUNT_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/sema/supported_strategy_margin_declaration.pine",
    "tests/fixtures/sema/unsupported_strategy_margin_declaration.pine",
    "tests/fixtures/runtime/strategy_margin_capital_held_long.pine",
    "tests/fixtures/runtime/strategy_margin_capital_held_short.pine",
    "tests/fixtures/runtime/strategy_margin_entry_affordability_long.pine",
    "tests/fixtures/runtime/strategy_margin_entry_affordability_short.pine",
    "tests/fixtures/runtime/strategy_margin_call_long.pine",
    "tests/fixtures/runtime/strategy_margin_call_short.pine",
];

const STRATEGY_MARGIN_STATE_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/strategy_margin_capital_held_long.pine",
    "tests/fixtures/runtime/strategy_margin_capital_held_short.pine",
    "tests/fixtures/runtime/strategy_margin_call_long.pine",
    "tests/fixtures/runtime/strategy_margin_call_short.pine",
];

const STRATEGY_CLOSE_ENTRIES_RULE_BOUNDARY_FIXTURES: &[&str] = &[
    "tests/fixtures/sema/supported_strategy_close_entries_rule_fifo.pine",
    "tests/fixtures/sema/supported_strategy_close_entries_rule_any.pine",
    "tests/fixtures/sema/unsupported_strategy_close_entries_rule_unknown.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_fifo.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_fifo_close_all.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_any_close.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_any_close_short.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_any_exit_from_entry.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_any_exit_from_entry_short.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_any_exit_same_id_partial.pine",
    "tests/fixtures/runtime/strategy_close_entries_rule_any_exit_same_id_partial_short.pine",
];

pub(super) fn validate_entry(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    validate_strategy_oca_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_execution_timing_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_process_orders_on_close_boundary_fixture_paths(
        line_number,
        feature,
        fixtures,
    )?;
    validate_strategy_calc_on_order_fills_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_calc_on_every_tick_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_bar_magnifier_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_risk_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_risk_allow_entry_in_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_risk_max_position_size_boundary_fixture_paths(
        line_number,
        feature,
        fixtures,
    )?;
    validate_strategy_risk_max_drawdown_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_risk_max_intraday_loss_boundary_fixture_paths(
        line_number,
        feature,
        fixtures,
    )?;
    validate_strategy_risk_max_intraday_filled_orders_boundary_fixture_paths(
        line_number,
        feature,
        fixtures,
    )?;
    validate_strategy_risk_max_cons_loss_days_boundary_fixture_paths(
        line_number,
        feature,
        fixtures,
    )?;
    validate_strategy_order_supported_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_close_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_close_all_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_cancel_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_short_reversal_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_max_contracts_held_short_boundary_fixture_paths(
        line_number,
        feature,
        fixtures,
    )?;
    validate_strategy_margin_account_boundary_fixture_paths(line_number, feature, fixtures)?;
    validate_strategy_close_entries_rule_boundary_fixture_paths(line_number, feature, fixtures)?;

    Ok(())
}

fn validate_strategy_oca_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    let required: Vec<&str> = match feature {
        "strategy constants" => vec![
            "tests/fixtures/runtime/strategy_constants.pine",
            "tests/fixtures/sema/unsupported_strategy_exit_oca_name_series.pine",
        ],
        "strategy.order" => STRATEGY_ORDER_OCA_BOUNDARY_FIXTURES.to_vec(),
        "strategy.entry" => STRATEGY_ENTRY_OCA_BOUNDARY_FIXTURES.to_vec(),
        "strategy.exit" => vec![
            "tests/fixtures/runtime/strategy_exit_oca_reduce.pine",
            "tests/fixtures/runtime/strategy_exit_oca_reduce_bracket.pine",
            "tests/fixtures/runtime/strategy_mixed_oca_order_exit_reduce.pine",
            "tests/fixtures/sema/supported_strategy_exit_oca_name.pine",
            "tests/fixtures/sema/unsupported_strategy_exit_oca_name_series.pine",
        ],
        "strategy.*" => STRATEGY_OCA_UNSUPPORTED_BOUNDARY_FIXTURES.to_vec(),
        _ => return Ok(()),
    };

    for fixture in required {
        if !fixtures.contains(&fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.order oca none/cancel/reduce, strategy.entry oca none/cancel/reduce, mixed-family OCA, and strategy.exit oca_name reduce are fixture-backed and series oca_name remains unsupported"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_execution_timing_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if !matches!(feature, "strategy" | "strategy.*") {
        return Ok(());
    }

    for fixture in STRATEGY_EXECUTION_TIMING_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy execution timing and recalculation settings remain unsupported"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_process_orders_on_close_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy" {
        return Ok(());
    }

    for fixture in STRATEGY_PROCESS_ORDERS_ON_CLOSE_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while const process_orders_on_close is an accepted execution-timing declaration setting"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_calc_on_order_fills_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy" {
        return Ok(());
    }

    for fixture in STRATEGY_CALC_ON_ORDER_FILLS_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while const calc_on_order_fills is an accepted historical recalculation setting"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_calc_on_every_tick_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy" {
        return Ok(());
    }

    for fixture in STRATEGY_CALC_ON_EVERY_TICK_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while const calc_on_every_tick is an accepted realtime forming-update setting"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_bar_magnifier_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy" {
        return Ok(());
    }

    for fixture in STRATEGY_BAR_MAGNIFIER_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while named const bool use_bar_magnifier is an accepted historical fill setting"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_risk_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.*" {
        return Ok(());
    }

    for fixture in STRATEGY_RISK_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while remaining strategy.risk broker directives stay unsupported"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_risk_max_drawdown_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.risk.max_drawdown" {
        return Ok(());
    }

    for fixture in STRATEGY_RISK_MAX_DRAWDOWN_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.risk.max_drawdown is the accepted drawdown stop rule"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_risk_max_intraday_loss_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.risk.max_intraday_loss" {
        return Ok(());
    }

    for fixture in STRATEGY_RISK_MAX_INTRADAY_LOSS_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.risk.max_intraday_loss is the accepted window loss stop"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_risk_max_cons_loss_days_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.risk.max_cons_loss_days" {
        return Ok(());
    }

    for fixture in STRATEGY_RISK_MAX_CONS_LOSS_DAYS_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.risk.max_cons_loss_days is the accepted consecutive-loss-day stop"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_risk_max_intraday_filled_orders_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.risk.max_intraday_filled_orders" {
        return Ok(());
    }

    for fixture in STRATEGY_RISK_MAX_INTRADAY_FILLED_ORDERS_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.risk.max_intraday_filled_orders is the accepted window fill-count stop"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_risk_max_position_size_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.risk.max_position_size" {
        return Ok(());
    }

    for fixture in STRATEGY_RISK_MAX_POSITION_SIZE_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.risk.max_position_size is the accepted entry-size risk rule"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_risk_allow_entry_in_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.risk.allow_entry_in" {
        return Ok(());
    }

    for fixture in STRATEGY_RISK_ALLOW_ENTRY_IN_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.risk.allow_entry_in is the accepted entry-direction risk rule"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_order_supported_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.order" {
        return Ok(());
    }

    for fixture in STRATEGY_ORDER_SUPPORTED_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.order support remains limited to the fixture-backed explicit-qty market signed-netting, limit/stop/stop-limit long, and limit/stop/stop-limit-short add-or-increase subset"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_close_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.close" {
        return Ok(());
    }

    for fixture in STRATEGY_CLOSE_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.close support remains limited to fixture-backed full closes, fixed/percent partial closes, flat/wrong-entry no-ops, and full-close pending-exit cleanup"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_close_all_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.close_all" {
        return Ok(());
    }

    for fixture in STRATEGY_CLOSE_ALL_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.close_all support remains limited to fixture-backed long-position closes, flat/repeated no-ops, metadata, const/simple immediately, and indicator/series-immediately rejections"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_cancel_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    let required: &[&str] = match feature {
        "strategy.cancel" => STRATEGY_CANCEL_BOUNDARY_FIXTURES,
        "strategy.cancel_all" => STRATEGY_CANCEL_ALL_BOUNDARY_FIXTURES,
        _ => return Ok(()),
    };

    for fixture in required {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy cancellation support remains limited to fixture-backed pending entry, generic-order, exit, and close cancellation without public cancellation records"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_short_reversal_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    let required: Vec<&str> = match feature {
        "strategy.entry" => STRATEGY_ENTRY_SHORT_POSITIVE_FIXTURES.to_vec(),
        "strategy.order" => STRATEGY_ORDER_SHORT_REVERSAL_BOUNDARY_FIXTURES.to_vec(),
        "strategy.exit" => STRATEGY_EXIT_SHORT_POSITIVE_FIXTURES.to_vec(),
        "strategy.*" => vec!["tests/fixtures/sema/unsupported_strategy_orders.pine"],
        _ => return Ok(()),
    };

    for fixture in required {
        if !fixtures.contains(&fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while short exposure and automatic reversal remain unsupported outside the fixture-backed market, limit, stop, and stop-limit strategy.entry short, market strategy.order signed netting, limit/stop/stop-limit strategy.order short add-or-increase, and short stop/limit/profit/loss/bracket/trailing strategy.exit subset"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_max_contracts_held_short_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "strategy.max_contracts_held_short" {
        return Ok(());
    }

    for fixture in STRATEGY_MAX_CONTRACTS_HELD_SHORT_BOUNDARY_FIXTURES {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy.max_contracts_held_short remains 0.0 in the long-only subset"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_margin_account_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    let required: &[&str] = match feature {
        "strategy" => STRATEGY_MARGIN_ACCOUNT_BOUNDARY_FIXTURES,
        "strategy.opentrades.*" | "strategy.opentrades.capital_held" => {
            STRATEGY_MARGIN_STATE_BOUNDARY_FIXTURES
        }
        "strategy.margin_liquidation_price" => &[
            "tests/fixtures/runtime/strategy_margin_call_long.pine",
            "tests/fixtures/runtime/strategy_margin_call_short.pine",
        ],
        _ => return Ok(()),
    };

    for fixture in required {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while strategy margin/account support remains limited to the fixture-backed long-only plus short capital-held/affordability/forced-liquidation/liquidation-price subset"
            ));
        }
    }

    Ok(())
}

fn validate_strategy_close_entries_rule_boundary_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    let required: &[&str] = match feature {
        "strategy" => STRATEGY_CLOSE_ENTRIES_RULE_BOUNDARY_FIXTURES,
        "strategy.*" => {
            &["tests/fixtures/sema/unsupported_strategy_close_entries_rule_unknown.pine"]
        }
        _ => return Ok(()),
    };

    for fixture in required {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` while close_entries_rule support remains limited to FIFO plus fixture-backed id-specific long and short ANY close/exit allocation"
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::super::try_conformance_entries_from_tsv;
    use super::*;

    #[test]
    fn rejects_strategy_star_row_without_oca_boundary_fixture_set() {
        let missing = "tests/fixtures/sema/unsupported_strategy_exit_oca_name_series.pine";
        let mut fixtures = vec!["tests/fixtures/sema/unsupported_strategy.pine"];
        fixtures.extend(
            STRATEGY_OCA_UNSUPPORTED_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.*\tunsupported\tcustom OCA parameters and strategy.order OCA behavior remain unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy OCA boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_row_without_execution_timing_boundary_fixture_set() {
        let missing = "tests/fixtures/sema/unsupported_strategy_declaration_properties.pine";
        let mut fixtures = vec![
            "tests/fixtures/runtime/strategy_no_order.pine",
            "tests/fixtures/sema/unsupported_strategy_pyramiding.pine",
        ];
        fixtures.extend(
            STRATEGY_EXECUTION_TIMING_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy\tpartial\tprocess_orders_on_close, calc_on_order_fills, calc_on_every_tick, use_bar_magnifier, and fill_orders_on_standard_ohlc remain unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy execution timing boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_star_row_without_risk_boundary_fixture_set() {
        let missing = "tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine";
        let mut fixtures = vec![
            "tests/fixtures/sema/unsupported_strategy.pine",
            "tests/fixtures/sema/unsupported_strategy_declaration_properties.pine",
        ];
        fixtures.extend(STRATEGY_OCA_UNSUPPORTED_BOUNDARY_FIXTURES.iter().copied());
        fixtures.extend(
            STRATEGY_RISK_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.*\tunsupported\tstrategy.risk broker directives remain unsupported until broker risk state is implemented\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy risk boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_allow_entry_in_row_without_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_risk_allow_entry_in_long.pine";
        let fixtures: Vec<&str> = STRATEGY_RISK_ALLOW_ENTRY_IN_BOUNDARY_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| *fixture != missing)
            .collect();

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.risk.allow_entry_in\tpartial\taccepted documented direction constants\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing allow_entry_in boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_max_position_size_row_without_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_risk_max_position_size_reduces.pine";
        let fixtures: Vec<&str> = STRATEGY_RISK_MAX_POSITION_SIZE_BOUNDARY_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| *fixture != missing)
            .collect();

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.risk.max_position_size\tpartial\treduces strategy.entry quantity\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing max_position_size boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_max_drawdown_row_without_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_risk_max_drawdown_cash.pine";
        let fixtures: Vec<&str> = STRATEGY_RISK_MAX_DRAWDOWN_BOUNDARY_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| *fixture != missing)
            .collect();

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.risk.max_drawdown\tpartial\tcancels pending, flattens, and blocks later trades\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing max_drawdown boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_max_intraday_loss_row_without_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_risk_max_intraday_loss_cash.pine";
        let fixtures: Vec<&str> = STRATEGY_RISK_MAX_INTRADAY_LOSS_BOUNDARY_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| *fixture != missing)
            .collect();

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.risk.max_intraday_loss\tpartial\tcancels pending, flattens, and blocks until window reset\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing max_intraday_loss boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_max_intraday_filled_orders_row_without_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_risk_max_intraday_filled_orders.pine";
        let fixtures: Vec<&str> = STRATEGY_RISK_MAX_INTRADAY_FILLED_ORDERS_BOUNDARY_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| *fixture != missing)
            .collect();

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.risk.max_intraday_filled_orders\tpartial\tcounts public fills then flattens until window reset\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing max_intraday_filled_orders boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_max_cons_loss_days_row_without_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_risk_max_cons_loss_days.pine";
        let fixtures: Vec<&str> = STRATEGY_RISK_MAX_CONS_LOSS_DAYS_BOUNDARY_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| *fixture != missing)
            .collect();

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.risk.max_cons_loss_days\tpartial\tpermanent stop after consecutive loss windows\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing max_cons_loss_days boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_order_row_without_supported_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_order_stop_limit_long.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_order_market_long.pine"];
        fixtures.extend(
            STRATEGY_ORDER_SUPPORTED_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.order\tpartial\texplicit-qty market, limit, stop, stop-limit long orders plus reduce-only market short orders are supported while omitted qty, short exposure, reversals, short price-based orders, OCA, and unsupported metadata types remain unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.order supported boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_cancel_row_without_pending_exit_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_cancel_exit.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_cancel_entry.pine"];
        fixtures.extend(
            STRATEGY_CANCEL_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.cancel\tpartial\tstrategy.cancel cancels matching pending entry and pending exit ids while filled or unknown ids are no-op\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.cancel pending-exit fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_cancel_row_without_noop_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_cancel_noop.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_cancel_entry.pine"];
        fixtures.extend(
            STRATEGY_CANCEL_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.cancel\tpartial\tstrategy.cancel cancels matching pending entry and pending exit ids while unknown, already-filled, or already-cancelled ids are no-op\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.cancel no-op fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_cancel_all_row_without_pending_exit_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_cancel_all_exit.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_cancel_all_entry_exit.pine"];
        fixtures.extend(
            STRATEGY_CANCEL_ALL_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.cancel_all\tpartial\tstrategy.cancel_all cancels supported pending entries and pending exits while empty pending books are no-op\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.cancel_all pending-exit fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_cancel_all_row_without_noop_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_cancel_all_noop.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_cancel_all_entry_exit.pine"];
        fixtures.extend(
            STRATEGY_CANCEL_ALL_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.cancel_all\tpartial\tstrategy.cancel_all cancels supported pending entries and pending exits while empty pending books are no-op\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.cancel_all no-op fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_close_all_row_without_noop_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_close_all.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_close_metadata.pine"];
        fixtures.extend(
            STRATEGY_CLOSE_ALL_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.close_all\tpartial\tstrategy.close_all closes current long positions while flat or already-closed calls are no-op and series immediately remains unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.close_all no-op fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_close_row_without_pending_exit_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_close_exit.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_close.pine"];
        fixtures.extend(
            STRATEGY_CLOSE_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.close\tpartial\tstrategy.close closes current long entries, supports partial quantities, treats wrong-entry ids as no-op, and clears matching pending exits only when the close fully flattens the entry\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.close pending-exit fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_close_all_row_without_pending_exit_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_close_all_exit.pine";
        let mut fixtures = vec!["tests/fixtures/runtime/strategy_close_all.pine"];
        fixtures.extend(
            STRATEGY_CLOSE_ALL_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.close_all\tpartial\tstrategy.close_all closes current long positions and clears pending exits for that entry while flat or already-closed calls are no-op\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy.close_all pending-exit fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_row_without_margin_account_boundary_fixture_set() {
        let missing = "tests/fixtures/runtime/strategy_margin_call_long.pine";
        let mut fixtures = vec![
            "tests/fixtures/runtime/strategy_no_order.pine",
            "tests/fixtures/sema/unsupported_strategy_declaration_properties.pine",
        ];
        fixtures.extend(
            STRATEGY_PROCESS_ORDERS_ON_CLOSE_BOUNDARY_FIXTURES
                .iter()
                .copied(),
        );
        fixtures.extend(
            STRATEGY_CALC_ON_ORDER_FILLS_BOUNDARY_FIXTURES
                .iter()
                .copied(),
        );
        fixtures.extend(
            STRATEGY_CALC_ON_EVERY_TICK_BOUNDARY_FIXTURES
                .iter()
                .copied(),
        );
        fixtures.extend(STRATEGY_BAR_MAGNIFIER_BOUNDARY_FIXTURES.iter().copied());
        fixtures.extend(
            STRATEGY_MARGIN_ACCOUNT_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy\tpartial\tmargin_long and margin_short declaration parsing is supported while runtime margin behavior remains long-only and short margin remains unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy margin/account boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_row_without_close_entries_rule_boundary_fixture_set() {
        let missing =
            "tests/fixtures/runtime/strategy_close_entries_rule_any_exit_same_id_partial.pine";
        let mut fixtures = vec![
            "tests/fixtures/runtime/strategy_no_order.pine",
            "tests/fixtures/sema/unsupported_strategy_declaration_properties.pine",
        ];
        fixtures.extend(
            STRATEGY_PROCESS_ORDERS_ON_CLOSE_BOUNDARY_FIXTURES
                .iter()
                .copied(),
        );
        fixtures.extend(
            STRATEGY_CALC_ON_ORDER_FILLS_BOUNDARY_FIXTURES
                .iter()
                .copied(),
        );
        fixtures.extend(
            STRATEGY_CALC_ON_EVERY_TICK_BOUNDARY_FIXTURES
                .iter()
                .copied(),
        );
        fixtures.extend(STRATEGY_BAR_MAGNIFIER_BOUNDARY_FIXTURES.iter().copied());
        fixtures.extend(STRATEGY_MARGIN_ACCOUNT_BOUNDARY_FIXTURES.iter().copied());
        fixtures.extend(
            STRATEGY_CLOSE_ENTRIES_RULE_BOUNDARY_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy\tpartial\tclose_entries_rule FIFO and fixture-backed id-specific ANY close and exit allocation are supported while broader ANY behavior remains unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing strategy close_entries_rule boundary fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_max_contracts_held_short_row_without_long_only_zero_fixture() {
        let missing = "tests/fixtures/runtime/strategy_position_state.pine";
        let tsv = "feature\tstatus\tnotes\tfixtures\nstrategy.max_contracts_held_short\tpartial\tremains 0.0 because short entries are unsupported\ttests/fixtures/sema/supported_strategy_position_state.pine;tests/fixtures/sema/unsupported_strategy_state_indicator.pine\n";
        let error = try_conformance_entries_from_tsv(tsv)
            .expect_err("missing long-only max_contracts_held_short fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_entry_row_without_market_short_runtime_fixture() {
        let missing = "tests/fixtures/runtime/strategy_entry_short.pine";
        let mut fixtures = vec![
            "tests/fixtures/runtime/strategy_entry.pine",
            "tests/fixtures/sema/unsupported_strategy_entry_qty.pine",
        ];
        fixtures.extend(STRATEGY_ENTRY_OCA_BOUNDARY_FIXTURES.iter().copied());
        fixtures.extend(
            STRATEGY_ENTRY_SHORT_POSITIVE_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.entry\tpartial\tmarket strategy.short entries are fixture-backed while reversal remains unsupported\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing market short strategy.entry runtime fixture should fail");

        assert!(error.contains(missing));
    }

    #[test]
    fn rejects_strategy_entry_row_without_stop_limit_short_runtime_fixture() {
        let missing = "tests/fixtures/runtime/strategy_entry_stop_limit_short.pine";
        let mut fixtures = vec![
            "tests/fixtures/runtime/strategy_entry.pine",
            "tests/fixtures/sema/unsupported_strategy_entry_qty.pine",
        ];
        fixtures.extend(STRATEGY_ENTRY_OCA_BOUNDARY_FIXTURES.iter().copied());
        fixtures.extend(
            STRATEGY_ENTRY_SHORT_POSITIVE_FIXTURES
                .iter()
                .copied()
                .filter(|fixture| *fixture != missing),
        );

        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstrategy.entry\tpartial\tstop-limit strategy.short entries are fixture-backed\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("missing stop-limit short strategy.entry runtime fixture should fail");

        assert!(error.contains(missing));
    }
}
