use super::BrokerState;
use pine_ir::StrategyCommission;

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 5.1e-9,
        "{actual} != {expected}"
    );
}

#[test]
fn native_foreign_account_ledger_uses_fill_rates_and_account_currency_fees() {
    for (fee, entry_fee, commission, profit, percent, held_equity) in [
        (
            StrategyCommission::Percent(1.),
            0.63899633,
            1.37689373,
            7.60591627,
            11.62168021,
            10008.30732867,
        ),
        (
            StrategyCommission::CashPerContract(1.),
            0.01,
            0.02,
            8.96281,
            13.82788417,
            10008.936325,
        ),
        (
            StrategyCommission::CashPerOrder(1.),
            1.,
            2.,
            6.98281,
            10.61105570,
            10007.946325,
        ),
    ] {
        let mut broker = BrokerState::new_with_commission(10000., Some(fee));
        broker.set_quote_to_account_rate(0.8874).unwrap();
        assert!(broker.entry_long("L".into(), 1, 1577923200000, 7200.77, 0.01));
        close(broker.open_trade_commission(0).unwrap(), entry_fee);
        close(broker.net_profit(), -entry_fee);
        broker.set_quote_to_account_rate(0.898).unwrap();
        close(broker.open_profit(8197.02), 8.946325);
        close(broker.equity_value(8197.02), held_equity);
        broker.set_quote_to_account_rate(0.9).unwrap();
        broker.close_long("L".into(), 10, 1578700800000, 8198.86);
        close(broker.closed_trade(0).unwrap().profit, profit);
        close(broker.closed_trade_commission(0).unwrap(), commission);
        close(broker.closed_trade_profit_percent(0).unwrap(), percent);
        close(broker.equity_value(8198.86), 10000. + profit);
        // Closed P/L is already in account currency; later rate changes must
        // neither revalue it nor change its recorded return percentage.
        broker.set_quote_to_account_rate(0.5).unwrap();
        close(broker.equity_value(8198.86), 10000. + profit);
        close(broker.closed_trade_profit_percent(0).unwrap(), percent);
    }
}

#[test]
fn unchanged_instrument_price_does_not_create_notional_fx_exposure() {
    for is_long in [true, false] {
        let mut broker = BrokerState::new(1000.);
        broker.set_quote_to_account_rate(0.8).unwrap();
        if is_long {
            assert!(broker.entry_long("L".into(), 0, 0, 100., 2.));
        } else {
            assert!(broker.entry_short("S".into(), 0, 0, 100., 2.));
        }
        for rate in [0.9, 0.5, 1.2, 0.8] {
            broker.set_quote_to_account_rate(rate).unwrap();
            assert_eq!(broker.equity_value(100.), 1000.);
            assert_eq!(broker.open_profit(100.), 0.);
            close(
                broker.open_profit(110.),
                if is_long { 20. * rate } else { -20. * rate },
            );
        }
    }
}

#[test]
fn invalid_rate_updates_leave_existing_broker_state_unchanged() {
    let mut broker = BrokerState::new(1000.);
    assert!(broker.entry_long("L".into(), 0, 0, 100., 2.));
    let before = broker.clone();
    for rate in [0., -1., f64::NAN, f64::INFINITY] {
        assert!(broker.set_quote_to_account_rate(rate).is_err());
        assert_eq!(broker, before);
    }
}

#[test]
fn different_entry_rates_and_fifo_partial_closes_preserve_account_commissions() {
    let mut broker = BrokerState::new_with_commission(1000., Some(StrategyCommission::Percent(1.)));
    broker.pyramiding_limit = 2;
    broker.set_quote_to_account_rate(0.8).unwrap();
    assert!(broker.entry_long("A".into(), 0, 0, 100., 1.));
    broker.set_quote_to_account_rate(0.9).unwrap();
    assert!(broker.entry_long("B".into(), 1, 1, 200., 1.));
    close(broker.equity_value(200.), 1087.4);
    broker.set_quote_to_account_rate(1.).unwrap();
    broker.close_long("A".into(), 2, 2, 300.);
    close(broker.closed_trade(0).unwrap().profit, 196.2);
    close(broker.open_trade_commission(0).unwrap(), 1.8);
    close(broker.equity_value(300.), 1294.4);
    broker.set_quote_to_account_rate(0.7).unwrap();
    broker.close_long("B".into(), 3, 3, 300.);
    close(broker.closed_trade(1).unwrap().profit, 66.1);
    close(broker.equity_value(300.), 1262.3);
}

#[test]
fn generic_cross_zero_fill_splits_one_account_order_fee_across_both_legs() {
    let mut broker =
        BrokerState::new_with_commission(1000., Some(StrategyCommission::CashPerOrder(1.)));
    broker.set_quote_to_account_rate(0.8).unwrap();
    assert!(broker.entry_long("L".into(), 0, 0, 100., 2.));
    broker.set_quote_to_account_rate(0.9).unwrap();
    assert!(broker.apply_generic_market_order_netting(
        "O".into(),
        -3.,
        1,
        1,
        110.,
        super::StrategyOrderMetadata::default(),
        false
    ));
    assert_eq!(broker.position_size, -1.);
    close(broker.closed_trade(0).unwrap().profit, 16.3333333333333);
    close(broker.open_trade_commission(0).unwrap(), 1. / 3.);
    close(broker.equity_value(110.), 1016.);
    broker.set_quote_to_account_rate(0.7).unwrap();
    close(broker.equity_value(100.), 1023.);
    broker.close_all_position(2, 2, 100.);
    close(broker.equity_value(100.), 1022.);
}

#[test]
fn nominal_full_close_preserves_fractional_ledger_residual_exposure() {
    let mut broker = BrokerState::new(5000.);
    broker.pyramiding_limit = 10;
    for (i, qty) in [
        0.11695922503796506,
        0.11814906993746994,
        0.11851687639144323,
        0.11965101345922058,
        0.11967553815466449,
        0.11913370419011882,
        0.11780946946605564,
        0.11779035326724228,
    ]
    .into_iter()
    .enumerate()
    {
        assert!(broker.entry_long("E".into(), i, i as i64, 100., qty));
    }
    broker.close_long("E".into(), 8, 8, 101.);
    let residual = broker.open_trade_size(0).unwrap();
    assert!(residual > 0. && residual < 1e-15, "{residual}");
    assert_eq!(broker.position_size, residual);
    broker.close_long("E".into(), 9, 9, 102.);
    assert_eq!(broker.position_size, 0.);
    assert!(broker.open_trade_size(0).is_none());
}
