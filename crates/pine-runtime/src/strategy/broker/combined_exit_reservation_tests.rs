use super::*;
use crate::runtime::strategy_path::{HistoricalPathKind, PathLeg};
use pine_ir::StrategyCloseEntriesRule;

fn broker(sizes: &[f64], rule: StrategyCloseEntriesRule) -> BrokerState {
    let mut broker = BrokerState::new(100_000.0);
    broker.pyramiding_limit = sizes.len() + 1;
    broker.close_entries_rule = rule;
    for &size in sizes {
        assert!(broker.entry_long("L".to_owned(), 0, 0, 100.0, size));
    }
    broker
}

fn place_protection(broker: &mut BrokerState, quantity: ExitQuantityRequest) {
    broker.place_combined_exit(
        "SL".to_owned(),
        String::new(),
        protection_spec(),
        quantity,
        1,
        StrategyExitMetadata::default(),
    );
}

fn protection_spec() -> CombinedExitSpec {
    CombinedExitSpec {
        stop: Some(90.0),
        limit: None,
        loss: None,
        profit: None,
        trail_price: Some(200.0),
        trail_points: None,
        trail_offset: Some(2.0),
        mintick: 1.0,
        compare_relative: true,
    }
}

fn place_generic(broker: &mut BrokerState, quantity: f64, percent: bool) {
    if percent {
        broker.place_exit_limit_qty_percent("TP".to_owned(), "L".to_owned(), 120.0, quantity, 1);
    } else {
        broker.place_exit_limit_qty("TP".to_owned(), "L".to_owned(), 120.0, quantity, 1);
    }
}

fn fill_at(broker: &mut BrokerState, price: f64) {
    while broker
        .take_next_realtime_event(EntryPathTick {
            bar_index: 2,
            time: 20,
            leg: PathLeg::point(price),
            path_kind: HistoricalPathKind::OpenHighLowClose,
            mark: price,
            long_blocked_at_path_start: false,
            short_blocked_at_path_start: false,
        })
        .is_some()
    {}
}

fn protection_quantities(broker: &BrokerState) -> Vec<f64> {
    broker
        .trade_ledger
        .open_trades()
        .iter()
        .map(|trade| {
            broker
                .order_book
                .exits()
                .find_by_identity_and_key("SL", &trade.id, Some(trade.key))
                .map_or(0.0, |exit| exit.reserved_quantity)
        })
        .collect()
}

#[test]
fn mixed_generic_and_combined_reservations_cover_each_trade_once() {
    for sizes in [&[1.0, 1.0][..], &[1.0, 2.0, 3.0], &[2.0, 0.5, 1.5]] {
        for rule in [
            StrategyCloseEntriesRule::Fifo,
            StrategyCloseEntriesRule::Any,
        ] {
            for percent in [false, true] {
                for generic_first in [false, true] {
                    for generic_fills_first in [false, true] {
                        let mut broker = broker(sizes, rule);
                        if !generic_first {
                            place_protection(&mut broker, ExitQuantityRequest::Percent(25.0));
                        }
                        let total: f64 = sizes.iter().sum();
                        place_generic(
                            &mut broker,
                            if percent { 25.0 } else { total / 4.0 },
                            percent,
                        );
                        place_protection(&mut broker, ExitQuantityRequest::Full);
                        let mut generic = total / 4.0;
                        let expected = sizes
                            .iter()
                            .map(|size| {
                                let used = generic.min(*size);
                                generic -= used;
                                size - used
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(
                            protection_quantities(&broker),
                            expected,
                            "{sizes:?} {rule:?} {percent} {generic_first}"
                        );
                        assert_eq!(
                            broker
                                .order_book
                                .exits()
                                .iter()
                                .map(|exit| exit.reserved_quantity)
                                .sum::<f64>(),
                            total
                        );
                        for price in if generic_fills_first {
                            [130.0, 89.0]
                        } else {
                            [89.0, 130.0]
                        } {
                            fill_at(&mut broker, price);
                        }
                        assert_eq!(broker.position_size, 0.0);
                        assert!(broker.diagnostics.is_empty(), "{:?}", broker.diagnostics);
                        broker.assert_ledger_aggregates();
                    }
                }
            }
        }
    }
}

#[test]
fn mixed_reservations_use_the_generic_fifo_or_any_trade_allocation() {
    for rule in [
        StrategyCloseEntriesRule::Fifo,
        StrategyCloseEntriesRule::Any,
    ] {
        let mut broker = broker(&[], rule);
        broker.pyramiding_limit = 3;
        assert!(broker.entry_long("old".to_owned(), 0, 0, 100.0, 2.0));
        assert!(broker.entry_long("L".to_owned(), 0, 0, 100.0, 1.0));
        assert!(broker.entry_long("L".to_owned(), 0, 0, 100.0, 2.0));
        place_generic(&mut broker, 25.0, true);
        place_protection(&mut broker, ExitQuantityRequest::Full);
        let expected = if rule == StrategyCloseEntriesRule::Fifo {
            vec![1.25, 1.0, 2.0]
        } else {
            vec![2.0, 0.25, 2.0]
        };
        assert_eq!(protection_quantities(&broker), expected, "{rule:?}");
        fill_at(&mut broker, 130.0);
        fill_at(&mut broker, 89.0);
        assert_eq!(broker.position_size, 0.0);
        assert!(broker.diagnostics.is_empty(), "{:?}", broker.diagnostics);
        broker.assert_ledger_aggregates();
    }
}

#[test]
fn mixed_reservations_share_only_the_matching_named_oca_group() {
    for same_group in [false, true] {
        let mut broker = broker(&[1.0, 2.0, 3.0], StrategyCloseEntriesRule::Any);
        broker.set_next_exit_oca_name(Some("generic".to_owned()));
        place_generic(&mut broker, 25.0, true);
        broker.set_next_exit_oca_name(Some(
            if same_group { "generic" } else { "combined" }.to_owned(),
        ));
        place_protection(&mut broker, ExitQuantityRequest::Full);
        assert_eq!(
            protection_quantities(&broker),
            if same_group {
                vec![1.0, 2.0, 3.0]
            } else {
                vec![0.0, 1.5, 3.0]
            }
        );
        assert!(broker.diagnostics.is_empty(), "{:?}", broker.diagnostics);
    }
}

#[test]
fn shared_peer_oca_reservation_is_counted_once_for_another_group() {
    for rule in [
        StrategyCloseEntriesRule::Fifo,
        StrategyCloseEntriesRule::Any,
    ] {
        for (take_profit_first, named_protection) in
            [(false, false), (true, false), (false, true), (true, true)]
        {
            let mut broker = broker(&[1.0, 1.0], rule);
            broker.set_next_exit_oca_name(Some("shared".to_owned()));
            place_generic(&mut broker, 25.0, true);
            broker.place_exit_stop_qty_percent(
                "shared-stop".to_owned(),
                "L".to_owned(),
                95.0,
                25.0,
                1,
            );
            assert_eq!(
                broker
                    .order_book
                    .exits()
                    .iter()
                    .map(|exit| exit.reserved_quantity)
                    .collect::<Vec<_>>(),
                vec![0.5, 0.5]
            );
            broker.set_next_exit_oca_name(named_protection.then(|| "protection".to_owned()));
            place_protection(&mut broker, ExitQuantityRequest::Full);
            assert_eq!(
                protection_quantities(&broker),
                vec![0.5, 1.0],
                "{rule:?}, shared peers must reserve their overlapping quantity once"
            );
            if take_profit_first {
                fill_at(&mut broker, 130.0);
            }
            fill_at(&mut broker, 89.0);
            // A named protection group also reduces its other trade's leg
            // after its first fill. Independent protection closes everything.
            assert_eq!(
                broker.position_size,
                if named_protection { 0.5 } else { 0.0 },
                "{rule:?} {take_profit_first} {named_protection}"
            );
            assert!(broker.diagnostics.is_empty(), "{:?}", broker.diagnostics);
            broker.assert_ledger_aggregates();
        }
    }
}

#[test]
fn shared_peer_oca_reservations_preserve_distinct_trade_footprints() {
    for rule in [
        StrategyCloseEntriesRule::Fifo,
        StrategyCloseEntriesRule::Any,
    ] {
        let mut broker = broker(&[2.0, 2.0], rule);
        let keys = broker
            .trade_ledger
            .open_trades()
            .iter()
            .map(|trade| trade.key)
            .collect::<Vec<_>>();
        broker.set_next_exit_oca_name(Some("shared".to_owned()));
        for (index, (key, quantity)) in keys.into_iter().zip([1.0, 0.5]).enumerate() {
            broker.place_combined_exit_for_trade(
                format!("peer-{index}"),
                key,
                protection_spec(),
                ExitQuantityRequest::Fixed(quantity),
                1,
                StrategyExitMetadata::default(),
            );
        }
        broker.set_next_exit_oca_name(Some("protection".to_owned()));
        place_protection(&mut broker, ExitQuantityRequest::Full);
        assert_eq!(protection_quantities(&broker), vec![1.0, 1.5], "{rule:?}");
        assert!(broker.diagnostics.is_empty(), "{:?}", broker.diagnostics);
    }
}
