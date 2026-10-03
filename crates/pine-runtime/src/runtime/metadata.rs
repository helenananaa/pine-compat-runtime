//! Immutable lookup tables shared by runtime checkpoints.
use std::collections::{HashMap, HashSet};

use pine_ir::{HirProgram, PersistenceKind, SeriesId, SymbolId, VarSlotId};

#[derive(Debug)]
pub(crate) struct RuntimeMetadata {
    pub(crate) calls: super::call_plan::CallPlan,
    symbols: HashMap<SymbolId, usize>,
    names: HashMap<String, usize>,
    history: HashSet<SeriesId>,
    execution_scoped: HashSet<SeriesId>,
    persistent_slots: [Vec<VarSlotId>; 3],
}

impl RuntimeMetadata {
    pub(crate) fn from_program(program: &HirProgram) -> Self {
        let mut metadata = Self {
            calls: super::call_plan::CallPlan::from_program(program),
            symbols: HashMap::with_capacity(program.symbols.len()),
            names: HashMap::with_capacity(program.symbols.len()),
            history: program
                .series_history
                .iter()
                .filter(|requirement| {
                    requirement.max_constant_offset > 0 || requirement.has_dynamic_offsets
                })
                .map(|requirement| requirement.series_id)
                .collect(),
            execution_scoped: program.execution_scoped_series.iter().copied().collect(),
            persistent_slots: Default::default(),
        };
        for (index, symbol) in program.symbols.iter().enumerate() {
            // Match the previous first-symbol lookup, including manually
            // constructed HIR with repeated display names or sparse IDs.
            metadata.symbols.entry(symbol.id).or_insert(index);
            metadata.names.entry(symbol.name.clone()).or_insert(index);
            if let Some(slot) = symbol.var_slot_id {
                metadata.persistent_slots[persistence_index(symbol.persistence)].push(slot);
            }
        }
        metadata
    }

    pub(crate) fn symbol_index(&self, symbol: SymbolId) -> Option<usize> {
        self.symbols.get(&symbol).copied()
    }

    pub(crate) fn named_symbol_index(&self, name: &str) -> Option<usize> {
        self.names.get(name).copied()
    }

    pub(crate) fn requires_history(&self, series: SeriesId) -> bool {
        self.history.contains(&series)
    }

    pub(crate) fn execution_scoped(&self, series: SeriesId) -> bool {
        self.execution_scoped.contains(&series)
    }

    pub(crate) fn persistent_slots(&self, kind: PersistenceKind) -> &[VarSlotId] {
        &self.persistent_slots[persistence_index(kind)]
    }
}

fn persistence_index(kind: PersistenceKind) -> usize {
    match kind {
        PersistenceKind::None => 0,
        PersistenceKind::Var => 1,
        PersistenceKind::Varip => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_tables_preserve_first_match_sparse_ids_and_history_requirements() {
        let source = pine_syntax::SourceFile::new(
            "metadata.pine",
            "//@version=6\nindicator(\"metadata\")\nvar saved = 1\nvarip intrabar = 2\nplot(close[bar_index % 3])\nplot(open[1])\n",
        );
        let analysis = pine_sema::analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let mut program = analysis.hir.unwrap();
        let mut duplicate = program.symbols[0].clone();
        duplicate.id = SymbolId(u32::MAX);
        program.symbols.push(duplicate.clone());
        duplicate.name = "sparse".to_owned();
        program.symbols.push(duplicate);
        let metadata = RuntimeMetadata::from_program(&program);
        for symbol in &program.symbols {
            assert_eq!(
                metadata.symbol_index(symbol.id),
                program
                    .symbols
                    .iter()
                    .position(|candidate| candidate.id == symbol.id)
            );
            assert_eq!(
                metadata.named_symbol_index(&symbol.name),
                program
                    .symbols
                    .iter()
                    .position(|candidate| candidate.name == symbol.name)
            );
        }
        assert_eq!(metadata.symbol_index(SymbolId(u32::MAX - 1)), None);
        assert_eq!(metadata.named_symbol_index("missing"), None);
        for id in 0..=program.next_series_id {
            let series = SeriesId(id);
            assert_eq!(
                metadata.requires_history(series),
                program
                    .series_history
                    .iter()
                    .any(|requirement| requirement.series_id == series
                        && (requirement.max_constant_offset > 0
                            || requirement.has_dynamic_offsets))
            );
        }
        for kind in [
            PersistenceKind::None,
            PersistenceKind::Var,
            PersistenceKind::Varip,
        ] {
            assert_eq!(
                metadata.persistent_slots(kind),
                program
                    .symbols
                    .iter()
                    .filter(|symbol| symbol.persistence == kind)
                    .filter_map(|symbol| symbol.var_slot_id)
                    .collect::<Vec<_>>()
            );
        }
    }
}
