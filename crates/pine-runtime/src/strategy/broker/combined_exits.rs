use super::{
    BrokerState, StrategyExitMetadata,
    ledger::TradeDirection,
    pending_exits::{
        DeferredRelativeExit, DeferredRelativeExitTrigger, ExitQuantityRequest, PendingExit,
        PendingExitTrigger, PendingTrailingActivation, PendingTrailingExit, PendingTrailingSpec,
        PendingTrailingState,
    },
    types::{InternalOrderKey, OcaMember},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CombinedExitSpec {
    pub(crate) stop: Option<f64>,
    pub(crate) limit: Option<f64>,
    pub(crate) loss: Option<f64>,
    pub(crate) profit: Option<f64>,
    pub(crate) trail_price: Option<f64>,
    pub(crate) trail_points: Option<f64>,
    pub(crate) trail_offset: Option<f64>,
    pub(crate) mintick: f64,
    pub(crate) compare_relative: bool,
}

impl BrokerState {
    pub(crate) fn place_combined_exit(
        &mut self,
        id: String,
        from_entry: String,
        spec: CombinedExitSpec,
        quantity: ExitQuantityRequest,
        bar_index: usize,
        metadata: StrategyExitMetadata,
    ) {
        if from_entry.is_empty() {
            let targets = (0..self.trade_ledger.open_count())
                .filter_map(|i| self.trade_ledger.open_at(i).map(|trade| trade.key))
                .collect::<Vec<_>>();
            for key in targets {
                self.place_combined_exit_for_trade(
                    id.clone(),
                    key,
                    spec,
                    quantity,
                    bar_index,
                    metadata.clone(),
                );
            }
            self.order_book
                .exits_mut()
                .replace_all_entry_deferred_relative(DeferredRelativeExit {
                    id,
                    from_entry,
                    trigger: DeferredRelativeExitTrigger::Combined(spec),
                    quantity,
                    last_update_bar_index: bar_index,
                    metadata,
                });
        } else if self.first_open_entry_price_for_entry(&from_entry).is_some() {
            let targets = (0..self.trade_ledger.open_count())
                .filter_map(|i| self.trade_ledger.open_at(i))
                .filter(|trade| trade.id == from_entry)
                .map(|trade| trade.key)
                .collect::<Vec<_>>();
            for key in targets {
                self.place_combined_exit_for_trade(
                    id.clone(),
                    key,
                    spec,
                    quantity,
                    bar_index,
                    metadata.clone(),
                );
            }
        } else if self
            .order_book
            .entries()
            .quantity_for_id(&from_entry)
            .is_some()
        {
            self.order_book
                .exits_mut()
                .replace_or_append_deferred_relative(DeferredRelativeExit {
                    id,
                    from_entry,
                    trigger: DeferredRelativeExitTrigger::Combined(spec),
                    quantity,
                    last_update_bar_index: bar_index,
                    metadata,
                });
        }
    }

    pub(super) fn place_combined_exit_for_trade(
        &mut self,
        id: String,
        key: u64,
        spec: CombinedExitSpec,
        quantity: ExitQuantityRequest,
        bar_index: usize,
        metadata: StrategyExitMetadata,
    ) {
        let Some((from_entry, price, size, direction)) = (0..self.trade_ledger.open_count())
            .filter_map(|i| self.trade_ledger.open_at(i))
            .find(|trade| trade.key == key)
            .map(|trade| {
                (
                    trade.id.clone(),
                    trade.entry_price,
                    trade.quantity,
                    trade.direction,
                )
            })
        else {
            return;
        };
        let Some(trigger) = self.resolve_combined_exit(spec, price, direction) else {
            return;
        };
        let oca_group = self.placing_exit_oca_group();
        let reserved = self.reserved_quantity_excluding_oca_group(
            &from_entry,
            Some((&id, &from_entry)),
            oca_group.as_ref(),
            Some(key),
        );
        if reserved >= size
            && match quantity {
                ExitQuantityRequest::Full => true,
                ExitQuantityRequest::Fixed(value) | ExitQuantityRequest::Percent(value) => {
                    value.is_finite() && value > 0.0
                }
            }
        {
            self.order_book
                .remove_exit_by_identity_and_key(&id, &from_entry, Some(key));
            return;
        }
        let Some((quantity, reserved_quantity)) = self.resolve_exit_quantity_request_for_available(
            quantity,
            size,
            (size - reserved).max(0.0),
        ) else {
            return;
        };
        if self
            .order_book
            .exits()
            .find_by_identity_and_key(&id, &from_entry, Some(key))
            .is_some_and(|exit| {
                exit.trigger.placement_equivalent(&trigger)
                    && exit.quantity == quantity
                    && exit.reserved_quantity == reserved_quantity
                    && exit.metadata == metadata
            })
        {
            return;
        }
        self.order_book.replace_or_append_exit(PendingExit {
            key: InternalOrderKey(0),
            id: id.clone(),
            from_entry: from_entry.clone(),
            target_trade_key: Some(key),
            trigger,
            quantity,
            reserved_quantity,
            multiple_reservation: true,
            last_update_bar_index: bar_index,
            metadata,
        });
        if let Some(group) = oca_group {
            self.order_book.assign_oca(
                OcaMember::Exit {
                    id,
                    from_entry,
                    target_trade_key: Some(key),
                },
                group,
            );
        }
    }

    fn resolve_combined_exit(
        &mut self,
        spec: CombinedExitSpec,
        entry_price: f64,
        direction: TradeDirection,
    ) -> Option<PendingExitTrigger> {
        let sign = if direction == TradeDirection::Long {
            1.0
        } else {
            -1.0
        };
        let mut relative = |ticks: Option<f64>, side: f64| -> Option<Option<f64>> {
            Some(match ticks {
                Some(ticks) => Some(
                    entry_price + side * sign * self.exit_tick_price_offset(ticks, spec.mintick)?,
                ),
                None => None,
            })
        };
        let stop = select_level(
            spec.stop,
            relative(spec.loss, -1.0)?,
            spec.compare_relative,
            direction == TradeDirection::Short,
        );
        let limit = select_level(
            spec.limit,
            relative(spec.profit, 1.0)?,
            spec.compare_relative,
            direction == TradeDirection::Long,
        );
        let activation = select_level(
            spec.trail_price,
            relative(spec.trail_points, 1.0)?,
            spec.compare_relative,
            direction == TradeDirection::Long,
        );
        let trigger = if let (Some(activation), Some(offset)) = (activation, spec.trail_offset) {
            PendingExitTrigger::Trailing(PendingTrailingExit {
                spec: PendingTrailingSpec {
                    fixed_stop: stop,
                    fixed_limit: limit,
                    activation: PendingTrailingActivation::Price(activation),
                    offset_price_distance: self.exit_tick_price_offset(offset, spec.mintick)?,
                },
                state: PendingTrailingState::Inactive,
            })
        } else {
            match (stop, limit) {
                (Some(downside), Some(upside)) => PendingExitTrigger::Bracket { downside, upside },
                (Some(price), None) => PendingExitTrigger::Stop(price),
                (None, Some(price)) => PendingExitTrigger::Limit(price),
                _ => return None,
            }
        };
        trigger.prices_are_finite().then_some(trigger)
    }
}

fn select_level(
    absolute: Option<f64>,
    relative: Option<f64>,
    compare: bool,
    lower: bool,
) -> Option<f64> {
    match (absolute, relative) {
        (Some(a), Some(r)) if compare => Some(if lower { a.min(r) } else { a.max(r) }),
        (Some(a), _) => Some(a),
        (_, value) => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::strategy_path::{HistoricalPathKind, PathLeg};
    use crate::strategy::EntryPathTick;
    use crate::strategy::broker::types::{OcaGroupKey, OcaMember, OcaType};

    fn spec() -> CombinedExitSpec {
        CombinedExitSpec {
            stop: Some(90.0),
            limit: Some(120.0),
            loss: None,
            profit: None,
            trail_price: Some(110.0),
            trail_points: None,
            trail_offset: Some(2.0),
            mintick: 1.0,
            compare_relative: true,
        }
    }

    #[test]
    fn combined_exit_all_entry_reservation_shares_named_oca_group() {
        let mut broker = BrokerState::new(100_000.0);
        assert!(broker.entry_long("L".to_owned(), 0, 10, 100.0, 2.0));
        broker.set_next_exit_oca_name(Some("g".to_owned()));
        broker.place_exit_limit_qty("TP".to_owned(), "L".to_owned(), 110.0, 1.0, 1);
        broker.place_combined_exit(
            "SL".to_owned(),
            String::new(),
            spec(),
            ExitQuantityRequest::Full,
            1,
            StrategyExitMetadata::default(),
        );
        assert_eq!(
            broker
                .order_book
                .exits()
                .find_by_identity("SL", "L")
                .map(|pending| pending.reserved_quantity),
            Some(2.0)
        );
        assert!(fill_observed_stop(&mut broker));
        assert_eq!(broker.position_size, 0.0);
        assert!(broker.diagnostics.is_empty());
        broker.assert_ledger_aggregates();
    }

    #[test]
    fn combined_exit_all_entry_oca_includes_each_pyramided_trade() {
        let mut broker = BrokerState::new(100_000.0);
        broker.pyramiding_limit = 2;
        assert!(broker.entry_long("L".to_owned(), 0, 10, 100.0, 1.0));
        assert!(broker.entry_long("L".to_owned(), 0, 10, 100.0, 1.0));
        broker.set_next_exit_oca_name(Some("g".to_owned()));
        broker.place_combined_exit(
            "SL".to_owned(),
            String::new(),
            spec(),
            ExitQuantityRequest::Full,
            1,
            StrategyExitMetadata::default(),
        );
        let group = OcaGroupKey::new("g", OcaType::Reduce);
        for pending in broker.order_book.exits().iter() {
            let member = OcaMember::Exit {
                id: pending.id.clone(),
                from_entry: pending.from_entry.clone(),
                target_trade_key: pending.target_trade_key,
            };
            assert_eq!(broker.order_book.oca_group(&member), Some(&group));
        }
        assert!(fill_observed_stop(&mut broker));
        assert_eq!(broker.position_size, 1.0);
        assert!(broker.diagnostics.is_empty());
        broker.assert_ledger_aggregates();
    }

    fn fill_observed_stop(broker: &mut BrokerState) -> bool {
        broker
            .take_next_realtime_event(EntryPathTick {
                bar_index: 2,
                time: 20,
                leg: PathLeg::point(89.0),
                path_kind: HistoricalPathKind::OpenHighLowClose,
                mark: 89.0,
                long_blocked_at_path_start: false,
                short_blocked_at_path_start: false,
            })
            .is_some_and(|outcome| outcome.is_fill())
    }
}
