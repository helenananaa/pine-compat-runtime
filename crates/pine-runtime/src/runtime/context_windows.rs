//! Dataset windows belong to a requested context, independently of expression
//! state. Dynamic Pine v5 retains its first demand; static requests and v6
//! can enlarge a shared window.
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{Bar, RequestKey};
use pine_ir::CallSiteId;

mod dependencies;
mod guards;
mod initial_dataset;
pub(crate) mod plan;
mod prepare;

#[derive(Clone, Debug, Default, PartialEq)]
struct Window {
    count: Option<usize>,
    since: Option<i64>,
    unbounded: bool,
    inherits_chart: bool,
    origin: Option<i64>,
}

impl Window {
    fn merge(&mut self, other: Self) {
        let before = (self.count, self.since, self.unbounded);
        self.unbounded |= other.unbounded;
        self.inherits_chart |= other.inherits_chart;
        self.count = self.count.into_iter().chain(other.count).max();
        self.since = self.since.into_iter().chain(other.since).min();
        if before != (self.count, self.since, self.unbounded) {
            self.origin = None;
        }
    }

    fn start(&mut self, bars: &[Bar]) -> usize {
        if self.unbounded || bars.is_empty() {
            return 0;
        }
        if let Some(origin) = self.origin {
            return bars.partition_point(|bar| bar.time < origin);
        }
        let start = self.initial_start(bars.len(), |time| {
            bars.partition_point(|bar| bar.time <= time)
        });
        self.origin = bars.get(start).map(|bar| bar.time);
        start
    }

    fn initial_start(&self, available: usize, at_or_before: impl FnOnce(i64) -> usize) -> usize {
        let count = self.count.map(|count| available.saturating_sub(count));
        let since = self.since.map(|time| at_or_before(time).saturating_sub(1));
        count.into_iter().chain(since).min().unwrap_or(0)
    }

    fn anchor_times(&mut self, times: &[i64]) {
        if !self.unbounded && self.origin.is_none() && !times.is_empty() {
            let start = self.initial_start(times.len(), |time| {
                times.partition_point(|stamp| *stamp <= time)
            });
            self.origin = times.get(start).copied();
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ContextWindows {
    pub(crate) prepared: bool,
    pub(crate) discovering: bool,
    pub(crate) restart: bool,
    dataset: Arc<[Bar]>,
    registrations: Arc<HashSet<(CallSiteId, RequestKey)>>,
    enlarge: bool,
    root: Option<RequestKey>,
    root_seen: bool,
    scope: Option<RequestKey>,
    root_count: Option<usize>,
    root_since: Option<i64>,
    windows: Arc<HashMap<RequestKey, Window>>,
}

impl ContextWindows {
    pub(crate) fn new(
        root: RequestKey,
        version: Option<u16>,
        dynamic: bool,
        count: Option<u32>,
    ) -> Self {
        Self {
            root: Some(root),
            enlarge: !dynamic || version.is_some_and(|version| version >= 6),
            root_count: count.filter(|count| *count > 0).map(|count| count as usize),
            ..Self::default()
        }
    }

    pub(crate) fn register_call(
        &mut self,
        id: CallSiteId,
        key: RequestKey,
        count: Option<usize>,
        bars: &[Bar],
    ) {
        if self.registrations.contains(&(id, key.clone())) {
            return;
        }
        Arc::make_mut(&mut self.registrations).insert((id, key.clone()));
        self.register(key, count, bars);
    }

    fn register(&mut self, key: RequestKey, count: Option<usize>, bars: &[Bar]) {
        let owned_bars = self.dataset.clone();
        let bars = if bars.is_empty() { &owned_bars } else { bars };
        if self.root.as_ref() == Some(&key) {
            let previous = self.root_count;
            if !self.root_seen {
                if let Some(count) = count {
                    self.root_count = Some(self.root_count.unwrap_or(0).max(count));
                }
                self.root_seen = true;
            } else if self.enlarge
                && let (Some(previous), Some(count)) = (self.root_count, count)
            {
                self.root_count = Some(previous.max(count));
            }
            self.update_root_since(bars);
            self.restart |= self.discovering && previous != self.root_count;
            return;
        }
        self.update_root_since(bars);
        let window = if let Some(count) = count {
            Window {
                count: Some(count),
                ..Window::default()
            }
        } else {
            let caller = self.scope.as_ref().and_then(|key| self.windows.get(key));
            Window {
                since: caller.map_or(self.root_since, |window| window.origin.or(window.since)),
                unbounded: caller.map_or(self.root_count.is_none(), |window| window.unbounded),
                inherits_chart: self.scope.is_none(),
                ..Window::default()
            }
        };
        match Arc::make_mut(&mut self.windows).entry(key) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(window);
            }
            std::collections::hash_map::Entry::Occupied(mut entry) if self.enlarge => {
                let previous = entry.get().clone();
                entry.get_mut().merge(window);
                self.restart |= self.discovering && *entry.get() != previous;
            }
            _ => {}
        }
    }

    fn update_root_since(&mut self, bars: &[Bar]) {
        if bars.is_empty() {
            return;
        }
        self.root_since = self.root_count.and_then(|count| {
            bars.get(bars.len().saturating_sub(count))
                .map(|bar| bar.time)
        });
    }

    pub(crate) fn initial_skip(&self, available: usize) -> usize {
        self.root_count
            .map_or(0, |count| available.saturating_sub(count))
    }

    pub(crate) fn is_root(&self, key: &RequestKey) -> bool {
        self.root.as_ref() == Some(key)
    }

    pub(crate) fn enter_scope(&mut self, key: RequestKey) {
        self.scope = Some(key);
        self.discovering = false;
        self.restart = false;
    }

    pub(crate) fn set_dataset(&mut self, bars: &[Bar]) {
        self.dataset = Arc::from(bars);
        self.update_root_since(bars);
    }

    pub(crate) fn finish(&mut self) {
        self.dataset = Arc::default();
        self.discovering = false;
        self.restart = false;
    }

    pub(crate) fn anchored_origin(&self, key: &RequestKey) -> Option<Option<i64>> {
        if self.root.as_ref() == Some(key) {
            return Some(self.root_since);
        }
        self.windows.get(key).and_then(|window| {
            if window.unbounded {
                Some(None)
            } else {
                window.origin.map(Some)
            }
        })
    }

    pub(crate) fn source_start(&mut self, key: &RequestKey, bars: &[Bar]) -> usize {
        if self.root.as_ref() == Some(key) {
            return self
                .root_since
                .map_or(0, |since| bars.partition_point(|bar| bar.time < since));
        }
        Arc::make_mut(&mut self.windows)
            .get_mut(key)
            .map_or(0, |window| window.start(bars))
    }

    pub(crate) fn anchor_known_history(&mut self, key: &RequestKey, times: &[i64]) {
        if let Some(window) = Arc::make_mut(&mut self.windows).get_mut(key) {
            window.anchor_times(times);
        }
    }
}
