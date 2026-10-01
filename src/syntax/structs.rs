use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, atomic::{AtomicU64, AtomicUsize}},
    time::{Duration},
};

use gpui_kit::{ElementId, Global, SharedString};

use crate::theme::Appearance;
use super::types::{LineRuns, MAX_ENTRIES};

/// One side of a diff as a whole text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SideText {
    pub text: String,
}

/// What a cached entry is for.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub(super) language: SharedString,
    pub(super) hash: u64,
    pub(super) appearance: Appearance,
}

impl Key {
    pub fn new(language: &str, text: &str, appearance: Appearance) -> Self {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        Self { language: SharedString::from(language.to_string()), hash: hasher.finish(), appearance }
    }
}

/// A slot's last highlighted text.
pub(super) struct Shown {
    pub(super) key: Key,
    pub(super) text: Arc<str>,
}

/// Highlighted texts, by language, content and theme. One per app.
#[derive(Default)]
pub struct SyntaxCache {
    pub(super) entries: HashMap<Key, Arc<Vec<LineRuns>>>,
    /// The entries' keys, oldest first.
    pub(super) order: VecDeque<Key>,
    pub(super) pending: HashSet<Key>,
    /// Each slot's last text that had its colours, kept as long as its entry.
    pub(super) shown: HashMap<ElementId, Shown>,
    /// Each slot's generation: it goes up with each text the slot asks to parse. At most
    /// [`MAX_ENTRIES`] slots, the oldest forgotten first.
    pub(super) generations: HashMap<ElementId, Arc<AtomicU64>>,
    /// The slots in `generations`, oldest first.
    pub(super) slots: VecDeque<ElementId>,
    /// How many background parses ran, for tests.
    pub parsed: Arc<AtomicUsize>,
    /// How many texts were parsed, for tests and for the bench.
    pub computed: usize,
    /// Time spent in [`highlight`] on the UI thread since the owner last reset it, for a frame log.
    pub spent: Duration,
}

impl Global for SyntaxCache {}

impl SyntaxCache {
    /// The entry for `key`, computing it with `compute` only the first time.
    pub fn get_or_compute(&mut self, key: Key, compute: impl FnOnce() -> Vec<LineRuns>) -> Arc<Vec<LineRuns>> {
        if let Some(hit) = self.entries.get(&key) {
            return hit.clone();
        }
        self.computed += 1;
        let lines = Arc::new(compute());
        if self.order.len() == MAX_ENTRIES
            && let Some(oldest) = self.order.pop_front()
        {
            self.entries.remove(&oldest);
            self.shown.retain(|_, shown| shown.key != oldest);
        }
        self.order.push_back(key.clone());
        self.entries.insert(key, lines.clone());
        lines
    }

    /// `slot`'s generation, a new one for a slot not seen lately.
    pub(super) fn generation(&mut self, slot: ElementId) -> Arc<AtomicU64> {
        if let Some(generation) = self.generations.get(&slot) {
            return generation.clone();
        }
        if self.slots.len() == MAX_ENTRIES
            && let Some(oldest) = self.slots.pop_front()
        {
            self.generations.remove(&oldest);
            self.shown.remove(&oldest);
        }
        self.slots.push_back(slot.clone());
        self.generations.entry(slot).or_default().clone()
    }

    pub(super) fn get(&self, key: &Key) -> Option<Arc<Vec<LineRuns>>> {
        self.entries.get(key).cloned()
    }
}
