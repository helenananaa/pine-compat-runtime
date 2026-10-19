//! Strategy pass rollback, including referenced heap state and varip overlays.
use super::*;

impl HistoricalRuntime<'_> {
    pub(super) fn snapshot_strategy_eval_checkpoint(&mut self) {
        self.strategy_eval_checkpoint = Some(StrategyEvalCheckpoint {
            ta_state: self.ta_state.clone(),
            valuewhen_local_values: self.valuewhen_budget.local_values,
            current_symbols: self.current_symbols.clone(),
            current_series: self.current_series.clone(),
            active_series: self.active_series.clone(),
            var_store: self.var_store.clone(),
            collections: CollectionCheckpoint::capture(self),
            price_flow: None,
        });
    }

    pub(super) fn restore_strategy_eval_checkpoint(&mut self) -> Result<(), RuntimeError> {
        let Some(checkpoint) = self.strategy_eval_checkpoint.take() else {
            return Ok(());
        };
        let previous = CollectionCheckpoint::capture(self);
        let roots = self
            .metadata
            .persistent_slots(PersistenceKind::Varip)
            .iter()
            .filter_map(|slot| self.var_store.get(slot).cloned())
            .collect();
        self.ta_state.restore_from(&checkpoint.ta_state);
        self.valuewhen_budget.local_values = checkpoint.valuewhen_local_values;
        self.current_symbols.clone_from(&checkpoint.current_symbols);
        self.current_series.clone_from(&checkpoint.current_series);
        self.active_series.clone_from(&checkpoint.active_series);
        // Historical fill recalculations rerun the script from the previous
        // committed bar. Ordinary `var` assignments from an earlier pass of
        // this bar must be discarded; `varip` deliberately survives passes.
        for slot in self
            .program
            .symbols
            .iter()
            .filter(|symbol| symbol.persistence == PersistenceKind::Var)
            .filter_map(|symbol| symbol.var_slot_id)
        {
            if let Some(value) = checkpoint.var_store.get(&slot) {
                self.var_store.insert(slot, value.clone());
            } else {
                self.var_store.remove(&slot);
            }
        }
        checkpoint.collections.restore(self);
        if let Some(flow) = &checkpoint.price_flow {
            flow.restore(self);
        }
        let result = self.seed_intrabar_objects_from_checkpoint(&previous, roots);
        self.strategy_eval_checkpoint = Some(checkpoint);
        result
    }
}
