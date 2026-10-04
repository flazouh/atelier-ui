use std::{collections::HashMap, time::Instant};

use gpui_kit::{
    App, AppContext, Context, ElementId, Entity, Global, IntoElement, Render, SharedString, Subscription, Task, Window,
};

use crate::{
    pr::{PrChipData, PrFacts},
    pr_card::LinkActions,
    pr_chip::PrOpenHandler,
};
use super::types::{CardActionHandler, CardOpenHandler, PrDoing, PrPart};

/// Which parts a card shows. Every part shows until the reader hides it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrParts(pub(super) u8);

impl Default for PrParts {
    fn default() -> Self {
        Self(PrPart::ALL.iter().fold(0, |bits, p| bits | p.bit()))
    }
}

impl PrParts {
    /// Every part but the ones named by key in `hidden`, as settings keep them.
    pub fn without(hidden: &[String]) -> Self {
        let mut parts = Self::default();
        for part in hidden.iter().filter_map(|k| PrPart::from_key(k)) {
            parts.set(part, false);
        }
        parts
    }

    pub fn shows(self, part: PrPart) -> bool {
        self.0 & part.bit() != 0
    }

    pub fn set(&mut self, part: PrPart, on: bool) {
        if on {
            self.0 |= part.bit();
        } else {
            self.0 &= !part.bit();
        }
    }

    /// The keys of the hidden parts, for settings.
    pub fn hidden(self) -> Vec<String> {
        PrPart::ALL.into_iter().filter(|p| !self.shows(*p)).map(|p| p.key().to_string()).collect()
    }
}

/// One changed file, as the card lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrFile {
    pub path: SharedString,
    pub added: u32,
    pub removed: u32,
}

/// The first failing check, and the first line of its log that says what went wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrFailing {
    pub name: SharedString,
    /// `None` while the log is read, or when no line says.
    pub line: Option<SharedString>,
    pub url: Option<SharedString>,
}

/// The session the pull request came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrSession {
    pub title: SharedString,
    pub status: SharedString,
    pub running: bool,
}

/// What the app read about one pull request once its card opened. Every field fills in on its own.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrGlance {
    /// Fresher than the chip's, once read.
    pub facts: Option<PrFacts>,
    /// The biggest changed files; `None` until read.
    pub files: Option<Vec<PrFile>>,
    pub failing: Option<PrFailing>,
    pub session: Option<PrSession>,
    pub doing: Option<PrDoing>,
    /// When the last read landed.
    pub read_at: Option<Instant>,
}

/// The pull request a glance is about.
pub type PrKey = (SharedString, u64);

/// Every card's shared state: the parts the reader shows, what the app read for each open card, and where an
/// open, a close or an action goes. A card observes it, so only open cards draw again when it changes.
#[derive(Default)]
pub struct PrCardStore {
    pub(super) parts: PrParts,
    pub(super) glances: HashMap<PrKey, PrGlance>,
    pub(crate) on_open: Option<CardOpenHandler>,
    pub(super) on_action: Option<CardActionHandler>,
}

impl PrCardStore {
    pub fn parts(&self) -> PrParts {
        self.parts
    }

    pub fn set_parts(&mut self, parts: PrParts, cx: &mut Context<Self>) {
        if self.parts != parts {
            self.parts = parts;
            cx.notify();
        }
    }

    pub fn glance(&self, key: &PrKey) -> Option<&PrGlance> {
        self.glances.get(key)
    }

    /// Changes the glance of `key`, made empty if there was none, and redraws its card if it changed.
    pub fn update_glance(&mut self, key: PrKey, change: impl FnOnce(&mut PrGlance), cx: &mut Context<Self>) {
        let glance = self.glances.entry(key).or_default();
        let before = glance.clone();
        change(glance);
        if *glance != before {
            cx.notify();
        }
    }

    /// `on_open` hears each card open (`true`) and close (`false`); it reads what the card shows.
    pub fn on_open(&mut self, f: impl Fn(&PrChipData, bool, &mut App) + 'static) {
        self.on_open = Some(std::rc::Rc::new(f));
    }

    pub fn on_action(&mut self, f: impl Fn(super::types::PrAction, &PrChipData, &mut Window, &mut App) + 'static) {
        self.on_action = Some(std::rc::Rc::new(f));
    }
}

pub(super) struct PrCards(pub(super) Entity<PrCardStore>);

impl Global for PrCards {}

/// One open card: it draws from its chip's data, then from what the app read since, and observes the store.
pub struct PrGlanceCard {
    pub(super) id: ElementId,
    pub(super) pr: PrChipData,
    pub(super) on_open: Option<PrOpenHandler>,
    pub(super) store: Entity<PrCardStore>,
    pub(super) menu: bool,
    /// Merge was pressed once: a second press within `CONFIRM_FOR` merges.
    pub(super) confirming: Option<Instant>,
    pub(super) actions: Entity<LinkActions>,
    pub(super) unconfirm: Task<()>,
    _store: Subscription,
}

impl PrGlanceCard {
    pub fn new(id: impl Into<ElementId>, pr: PrChipData, on_open: Option<PrOpenHandler>, cx: &mut Context<Self>) -> Self {
        let store = super::helpers::pr_cards(cx);
        let _store = cx.observe(&store, |_, _, cx| cx.notify());
        let actions = cx.new(|_| LinkActions::default());
        Self { id: id.into(), pr, on_open, store, menu: false, confirming: None, actions, unconfirm: Task::ready(()), _store }
    }

    /// The chip's newest data, as the text that holds it draws again.
    pub fn set_pr(&mut self, pr: PrChipData, cx: &mut Context<Self>) {
        if self.pr != pr {
            self.pr = pr;
            cx.notify();
        }
    }
}

impl Render for PrGlanceCard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        super::helpers::card(self, window, cx)
    }
}
