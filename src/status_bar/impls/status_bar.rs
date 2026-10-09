use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Styled, Window, div,
};

use super::super::{
    helpers::{load_parts, usage_part, version_part, work_part},
    structs::{ProviderGauge, StatusBar, StatusCard, StatusRow, SystemLoad, Work},
};

impl StatusBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            load: None,
            work: Work::default(),
            providers: Vec::new(),
            lead: None,
            tail: None,
            version: None,
            on_version: None,
            on_usage: None,
        }
    }

    pub fn load(mut self, load: Option<SystemLoad>) -> Self {
        self.load = load;
        self
    }

    pub fn work(mut self, work: Work) -> Self {
        self.work = work;
        self
    }

    pub fn providers(mut self, providers: Vec<ProviderGauge>) -> Self {
        self.providers = providers;
        self
    }

    /// The app's version, a small mono label at the left of the bar. A press on it fires `on_version`.
    pub fn version(mut self, version: Option<SharedString>) -> Self {
        self.version = version;
        self
    }

    /// A press on the version: the app shows what is new in it. The label is pressable only with this.
    pub fn on_version(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_version = Some(Rc::new(f));
        self
    }

    /// A press on the provider chips, taken as one target: the app opens the usage. Each chip keeps its own tooltip.
    pub fn on_usage(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_usage = Some(Rc::new(f));
        self
    }

    /// The bar's cards stand under the columns above it, left to right: the version, the work and the providers'
    /// usage, then the processor and the memory.
    ///
    /// `lead` is the width of the card under the sidebar (the version goes there) and `tail` of the card under the
    /// right pane (the processor and the memory go there). The card between them takes the rest. With no `lead` the
    /// version stands first in the middle card, and with no `tail` the processor and the memory stand last in it, so
    /// there is one card. A card of a given width cuts what does not fit and never spills into the next.
    pub fn columns(mut self, lead: Option<f32>, tail: Option<f32>) -> Self {
        self.lead = lead;
        self.tail = tail;
        self
    }
}

impl RenderOnce for StatusBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let machine = self
            .load
            .as_ref()
            .map(|load| load_parts(&self.id, load, cx))
            .unwrap_or_default();
        let work = work_part(self.work, cx);
        let usage = usage_part(&self.id, &self.providers, self.on_usage.clone(), window, cx);
        let version = self
            .version
            .clone()
            .map(|version| version_part(&self.id, version, self.on_version.clone(), window, cx));
        let (lead, tail) = (self.lead, self.tail.filter(|_| self.load.is_some()));
        let mut cards: Vec<StatusCard> = Vec::new();
        let mut middle: Vec<AnyElement> = Vec::new();
        // The version has the sidebar's card to itself; with no column to stand under, it leads the middle card.
        match lead {
            Some(width) => cards.push(
                StatusCard::new("status-card-version")
                    .width(Some(width))
                    .children(version),
            ),
            None => middle.extend(version),
        }
        middle.extend(work);
        middle.extend(usage);
        match tail {
            Some(width) => {
                cards.push(StatusCard::new("status-card-main").children(middle));
                cards.push(
                    StatusCard::new("status-card-load")
                        .width(Some(width))
                        .children(machine),
                );
            }
            None => {
                middle.push(div().flex_1().into_any_element());
                middle.extend(machine);
                cards.push(StatusCard::new("status-card-main").children(middle));
            }
        }
        StatusRow::new(self.id.clone()).children(cards)
    }
}
