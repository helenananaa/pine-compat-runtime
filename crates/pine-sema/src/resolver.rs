use std::collections::HashMap;
use std::collections::hash_map::Entry;

use pine_ir::{HirSymbol, PersistenceKind, PineType, SeriesId, SymbolId, VarSlotId};
use pine_syntax::Span;

use crate::source_graph::SourceContextId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SymbolInfo {
    pub(crate) id: SymbolId,
    pub(crate) pine_type: PineType,
    pub(crate) series_id: Option<SeriesId>,
    pub(crate) persistence: PersistenceKind,
    pub(crate) var_slot_id: Option<VarSlotId>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BindingKey {
    pub(crate) source_context_id: SourceContextId,
    pub(crate) span_start: usize,
    pub(crate) span_end: usize,
    pub(crate) name: String,
}
/// Symbols bound to source tokens. Each symbol id keeps the slots bound to it,
/// so updating one symbol does not scan every binding in the program.
#[derive(Debug, Clone, Default)]
pub(crate) struct Bindings {
    symbols: Vec<SymbolInfo>,
    slots: HashMap<BindingKey, usize>,
    slots_by_symbol: HashMap<SymbolId, Vec<usize>>,
}
impl Bindings {
    pub(crate) fn get(&self, key: &BindingKey) -> Option<&SymbolInfo> {
        self.slots.get(key).map(|&slot| &self.symbols[slot])
    }

    pub(crate) fn insert(&mut self, key: BindingKey, symbol: SymbolInfo) {
        let slot = match self.slots.entry(key) {
            Entry::Occupied(entry) => {
                let slot = *entry.get();
                let previous = std::mem::replace(&mut self.symbols[slot], symbol);
                if previous.id == symbol.id {
                    return;
                }
                slot
            }
            Entry::Vacant(entry) => {
                let slot = self.symbols.len();
                self.symbols.push(symbol);
                entry.insert(slot);
                slot
            }
        };
        self.slots_by_symbol
            .entry(symbol.id)
            .or_default()
            .push(slot);
    }

    /// Replaces every binding of `symbol.id` with `symbol`.
    pub(crate) fn update_symbol(&mut self, symbol: SymbolInfo) {
        let Some(slots) = self.slots_by_symbol.get(&symbol.id) else {
            return;
        };
        for &slot in slots {
            // A slot later rebound to another symbol keeps its old index entry.
            if self.symbols[slot].id == symbol.id {
                self.symbols[slot] = symbol;
            }
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct ScopeResolver {
    pub(crate) scopes: Vec<HashMap<String, SymbolInfo>>,
    all_symbols: Vec<(String, SymbolInfo)>,
    // Position of each symbol id's first `all_symbols` entry.
    lower_symbol_positions: HashMap<SymbolId, usize>,
}
impl ScopeResolver {
    pub(crate) fn new(
        global_symbols: HashMap<String, SymbolInfo>,
        symbol_order: Vec<String>,
    ) -> Self {
        let mut resolver = Self {
            scopes: Vec::new(),
            all_symbols: Vec::new(),
            lower_symbol_positions: HashMap::new(),
        };
        for name in &symbol_order {
            if let Some(&symbol) = global_symbols.get(name) {
                resolver.push_lower_symbol(name, symbol);
            }
        }
        resolver.scopes.push(global_symbols);
        resolver
    }

    pub(crate) fn all_symbols(&self) -> &[(String, SymbolInfo)] {
        &self.all_symbols
    }

    fn push_lower_symbol(&mut self, name: &str, info: SymbolInfo) {
        self.lower_symbol_positions
            .entry(info.id)
            .or_insert(self.all_symbols.len());
        self.all_symbols.push((name.to_owned(), info));
    }

    pub(crate) fn resolve(&self, name: &str) -> Option<SymbolInfo> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    pub(crate) fn resolves_to_global(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rposition(|scope| scope.contains_key(name))
            .is_some_and(|index| index == 0)
    }

    pub(crate) fn symbol_is_global(&self, id: SymbolId) -> bool {
        self.scopes
            .first()
            .is_some_and(|scope| scope.values().any(|symbol| symbol.id == id))
    }

    pub(crate) fn define_global(&mut self, name: &str, info: SymbolInfo) {
        let global_scope = self
            .scopes
            .first_mut()
            .expect("scope resolver always has a global scope");
        if global_scope.insert(name.to_owned(), info).is_none() {
            self.push_lower_symbol(name, info);
        } else {
            self.update_lower_symbol(info);
        }
    }

    pub(crate) fn update(&mut self, name: &str, info: SymbolInfo) {
        if let Some(scope) = self
            .scopes
            .iter_mut()
            .rev()
            .find(|scope| scope.contains_key(name))
        {
            scope.insert(name.to_owned(), info);
        }
        self.update_lower_symbol(info);
    }

    pub(crate) fn update_lower_symbol(&mut self, info: SymbolInfo) {
        if let Some(&position) = self.lower_symbol_positions.get(&info.id) {
            self.all_symbols[position].1 = info;
        }
    }

    pub(crate) fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub(crate) fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub(crate) fn define_local(&mut self, name: &str, info: SymbolInfo, lower: bool) {
        let scope = self
            .scopes
            .last_mut()
            .expect("scope resolver always has a current scope");
        scope.insert(name.to_owned(), info);
        if lower {
            self.push_lower_symbol(name, info);
        }
    }

    pub(crate) fn lower_symbols(&self) -> Vec<HirSymbol> {
        self.all_symbols
            .iter()
            .map(|(name, symbol)| HirSymbol {
                id: symbol.id,
                name: name.clone(),
                pine_type: symbol.pine_type,
                series_id: symbol.series_id,
                persistence: symbol.persistence,
                var_slot_id: symbol.var_slot_id,
            })
            .collect()
    }

    pub(crate) fn contains_lower_symbol(&self, id: SymbolId) -> bool {
        self.lower_symbol_positions.contains_key(&id)
    }

    pub(crate) fn add_lower_symbol(&mut self, name: &str, info: SymbolInfo) {
        self.push_lower_symbol(name, info);
    }
}
pub(crate) fn binding_key(
    source_context_id: SourceContextId,
    name: &str,
    span: Span,
) -> BindingKey {
    BindingKey {
        source_context_id,
        span_start: span.start,
        span_end: span.end,
        name: name.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use pine_ir::{Qualifier, ValueKind};

    use super::*;

    fn symbol(id: u32, series_id: u32) -> SymbolInfo {
        SymbolInfo {
            id: SymbolId(id),
            pine_type: PineType::new(Qualifier::Series, ValueKind::Float),
            series_id: Some(SeriesId(series_id)),
            persistence: PersistenceKind::None,
            var_slot_id: None,
        }
    }

    fn key(name: &str, start: usize) -> BindingKey {
        binding_key(SourceContextId::root(), name, Span::new(start, start + 1))
    }

    #[test]
    fn symbol_update_rewrites_only_its_current_bindings() {
        let mut bindings = Bindings::default();
        bindings.insert(key("x", 0), symbol(1, 10));
        bindings.insert(key("x", 4), symbol(1, 10));
        bindings.insert(key("y", 8), symbol(2, 20));
        // Rebinding a key moves it to the new symbol; the old index entry goes stale.
        bindings.insert(key("x", 4), symbol(2, 20));
        bindings.insert(key("x", 12), symbol(2, 20));
        bindings.insert(key("x", 12), symbol(1, 10));

        bindings.update_symbol(symbol(1, 11));

        assert_eq!(bindings.get(&key("x", 0)), Some(&symbol(1, 11)));
        assert_eq!(bindings.get(&key("x", 4)), Some(&symbol(2, 20)));
        assert_eq!(bindings.get(&key("y", 8)), Some(&symbol(2, 20)));
        assert_eq!(bindings.get(&key("x", 12)), Some(&symbol(1, 11)));

        bindings.update_symbol(symbol(2, 21));

        assert_eq!(bindings.get(&key("x", 4)), Some(&symbol(2, 21)));
        assert_eq!(bindings.get(&key("x", 12)), Some(&symbol(1, 11)));
        assert_eq!(bindings.get(&key("missing", 0)), None);
    }

    #[test]
    fn lower_symbol_updates_follow_the_first_entry_for_an_id() {
        let mut resolver = ScopeResolver::new(HashMap::new(), Vec::new());
        resolver.define_global("x", symbol(1, 10));
        resolver.add_lower_symbol("x_copy", symbol(1, 10));
        resolver.define_local("y", symbol(2, 20), false);

        resolver.update_lower_symbol(symbol(1, 11));
        resolver.define_global("x", symbol(1, 12));

        assert!(resolver.contains_lower_symbol(SymbolId(1)));
        assert!(!resolver.contains_lower_symbol(SymbolId(2)));
        assert_eq!(
            resolver.all_symbols(),
            [
                ("x".to_owned(), symbol(1, 12)),
                ("x_copy".to_owned(), symbol(1, 10)),
            ]
        );
    }
}
