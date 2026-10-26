use super::*;

fn normalize_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

fn count(broker: &BrokerState, include: impl Fn(f64) -> bool) -> i64 {
    i64::try_from(
        broker
            .trades
            .iter()
            .filter(|trade| include(trade.profit))
            .count(),
    )
    .expect("trade count")
}

fn scanned_average(
    broker: &BrokerState,
    include: impl Fn(f64) -> bool,
    value: impl Fn(&StrategyTrade, &ClosedTradeMetrics) -> f64,
) -> Option<f64> {
    let mut count = 0usize;
    let mut total = 0.0;
    for (trade, metrics) in broker.trades.iter().zip(&broker.closed_trade_metrics) {
        if include(trade.profit) {
            count += 1;
            total += value(trade, metrics);
        }
    }
    (count > 0).then(|| normalize_zero(total / count as f64))
}

fn assert_same(name: &str, cached: Option<f64>, scanned: Option<f64>) {
    assert_eq!(
        cached.map(f64::to_bits),
        scanned.map(f64::to_bits),
        "{name}: cached {cached:?}, scanned {scanned:?}"
    );
}

/// The cached totals must be bit-identical to scanning every closed trade.
fn assert_statistics_match_scan(broker: &BrokerState) {
    assert_eq!(broker.winning_trade_count(), count(broker, |p| p > 0.0));
    assert_eq!(broker.losing_trade_count(), count(broker, |p| p < 0.0));
    assert_eq!(
        broker.even_trade_count(),
        count(broker, |p| normalize_zero(p) == 0.0)
    );
    let gross_profit = broker
        .trades
        .iter()
        .filter(|trade| trade.profit > 0.0)
        .map(|trade| trade.profit)
        .sum();
    let gross_loss = broker
        .trades
        .iter()
        .filter(|trade| trade.profit < 0.0)
        .map(|trade| -trade.profit)
        .sum();
    assert_same(
        "gross profit",
        Some(broker.gross_profit()),
        Some(normalize_zero(gross_profit)),
    );
    assert_same(
        "gross loss",
        Some(broker.gross_loss()),
        Some(normalize_zero(gross_loss)),
    );
    assert_same(
        "average winning trade",
        broker.average_winning_trade(),
        scanned_average(broker, |p| p > 0.0, |trade, _| trade.profit),
    );
    assert_same(
        "average losing trade",
        broker.average_losing_trade(),
        scanned_average(broker, |p| p < 0.0, |trade, _| -trade.profit),
    );
    assert_same(
        "average trade percent",
        broker.average_trade_percent(),
        scanned_average(broker, |_| true, |_, metrics| metrics.profit_percent),
    );
    assert_same(
        "average winning trade percent",
        broker.average_winning_trade_percent(),
        scanned_average(broker, |p| p > 0.0, |_, metrics| metrics.profit_percent),
    );
    assert_same(
        "average losing trade percent",
        broker.average_losing_trade_percent(),
        scanned_average(broker, |p| p < 0.0, |_, metrics| -metrics.profit_percent),
    );
}

#[test]
fn closed_trade_statistics_follow_partial_closes_fees_and_checkpoints() {
    let mut broker =
        BrokerState::new_with_commission(100_000.0, Some(StrategyCommission::Percent(0.1)));
    assert_statistics_match_scan(&broker);
    assert_eq!(broker.average_trade_percent(), None);

    assert!(broker.entry_long("L".to_owned(), 0, 10, 100.0, 3.0));
    broker.close_long_qty("L".to_owned(), 1, 20, 110.0, 1.0);
    assert_eq!(broker.trades.len(), 1);
    assert_statistics_match_scan(&broker);
    let checkpoint = broker.snapshot();

    broker.close_long("L".to_owned(), 2, 30, 95.0);
    assert!(broker.entry_short("S".to_owned(), 3, 40, 100.0, 2.0));
    broker.close_long_qty("S".to_owned(), 4, 50, 90.0, 0.5);
    broker.close_long("S".to_owned(), 5, 60, 104.0);
    // A flat round trip loses only its fees.
    assert!(broker.entry_long("F".to_owned(), 6, 70, 100.0, 1.0));
    broker.close_long("F".to_owned(), 7, 80, 100.0);
    assert_eq!(broker.trades.len(), 5);
    assert_eq!(broker.winning_trade_count(), 2);
    assert_eq!(broker.losing_trade_count(), 3);
    assert_statistics_match_scan(&broker);

    broker.restore(checkpoint);
    assert_eq!(broker.trades.len(), 1);
    assert_eq!(broker.winning_trade_count(), 1);
    assert_eq!(broker.losing_trade_count(), 0);
    assert_statistics_match_scan(&broker);
}

#[test]
fn closed_trade_totals_keep_the_scan_fold_order_and_signed_zero_rules() {
    // Magnitudes chosen so a different summation order changes the result.
    let trades = [
        (1.0e16, 3.0),
        (1.0, 0.1),
        (-1.0e16, -3.0),
        (-0.0, 0.0),
        (0.0, -0.0),
        (-1.0, -0.2),
        // A non-finite profit has a zero profit percent.
        (f64::NAN, 0.0),
        (f64::MIN_POSITIVE, 1.0e-300),
        (-3.0, -0.3),
        (7.0, 0.7),
    ];
    let mut stats = ClosedTradeStats::default();
    for (index, (profit, percent)) in trades.into_iter().enumerate() {
        stats.record(profit, percent);
        let seen = &trades[..=index];
        let sum = |include: fn(f64) -> bool, sign: f64, field: fn(&(f64, f64)) -> f64| {
            seen.iter()
                .filter(|trade| include(trade.0))
                .map(|trade| sign * field(trade))
                .sum::<f64>()
        };
        let total = |include: fn(f64) -> bool| seen.iter().filter(|trade| include(trade.0)).count();
        let wins = |profit: f64| profit > 0.0;
        let losses = |profit: f64| profit < 0.0;
        assert_eq!(stats.winning_count(), total(wins));
        assert_eq!(stats.losing_count(), total(losses));
        assert_eq!(stats.even_count(), total(|profit| profit == 0.0));
        assert_eq!(
            normalize_zero(stats.gross_profit()).to_bits(),
            normalize_zero(sum(wins, 1.0, |trade| trade.0)).to_bits()
        );
        assert_eq!(
            normalize_zero(stats.gross_loss()).to_bits(),
            normalize_zero(sum(losses, -1.0, |trade| trade.0)).to_bits()
        );
        let mut percent_total = 0.0;
        for trade in seen {
            percent_total += trade.1;
        }
        assert_same(
            "average trade percent",
            stats.average_trade_percent(),
            Some(percent_total / seen.len() as f64),
        );
    }
}
