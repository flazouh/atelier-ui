use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::{
    menu::lead_icon,
    panel_layout::GAP,
    scale::px,
    theme::{ActiveTheme, Theme, radius},
    tooltip::Tooltip,
    typography::{MONO_FONT_FAMILY, TextSize},
};

use super::super::{
    consts::{GAUGE_HEIGHT, GAUGE_WIDTH, HEIGHT, MARK, MINI_WIDTH, SPARK_BARS, SPARK_GAP, SPARK_HEIGHT, SPARK_WIDTH},
    enums::{GaugeState, Pressure},
    helpers::press_target,
    structs::{ProviderGauge, StatusBar, SystemLoad, Work},
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

    fn part(&self, name: &'static str) -> ElementId {
        ElementId::from((self.id.clone(), name))
    }
}

/// One card of the bar, as a panel is one: the card tone, the panels' corners and the bar's whole height. `width` is the width of
/// the column above it; none takes what the others leave.
fn card(items: Vec<AnyElement>, debug: &'static str, width: Option<f32>, theme: &Theme) -> AnyElement {
    div()
        .debug_selector(move || debug.into())
        .flex()
        .items_center()
        .gap(px(10.))
        .h_full()
        .px(px(10.))
        .rounded(radius::lg())
        .bg(theme.card)
        .overflow_hidden()
        .when_some(width, |d, w| d.flex_none().w(px(w)))
        .when(width.is_none(), |d| d.flex_1().min_w_0())
        .children(items)
        .into_any_element()
}

/// A caption and what it measures, with a hover.
fn cluster(id: ElementId, debug: &'static str, tooltip: String, theme: &Theme) -> gpui_kit::Stateful<gpui_kit::Div> {
    div()
        .id(id)
        .debug_selector(move || debug.into())
        .flex()
        .flex_none()
        .items_center()
        .gap(px(6.))
        .text_color(theme.muted_foreground)
        .tooltip(Tooltip::text(tooltip))
}

/// The processor's last samples, newest at the right, each in the colour of its own pressure and dimmer the older it is.
fn spark(history: &[f32], theme: &Theme) -> AnyElement {
    let shown: Vec<Option<f32>> = {
        let samples = history
            .iter()
            .rev()
            .take(SPARK_BARS)
            .rev()
            .map(|s| Some(s.clamp(0., 1.)));
        std::iter::repeat_n(None, SPARK_BARS.saturating_sub(history.len()))
            .chain(samples)
            .collect()
    };
    div()
        .flex()
        .flex_none()
        .items_end()
        .gap(px(SPARK_GAP))
        .h(px(SPARK_HEIGHT))
        .children(shown.iter().enumerate().map(|(at, sample)| {
            let age = (at + 1) as f32 / SPARK_BARS as f32;
            let (height, ink) = match sample {
                Some(used) => (
                    1. + (SPARK_HEIGHT - 1.) * used,
                    Pressure::of(*used).ink(theme).opacity(0.3 + 0.7 * age),
                ),
                None => (1., theme.muted_foreground.opacity(0.15)),
            };
            div().w(px(SPARK_WIDTH)).h(px(height)).rounded(px(1.)).bg(ink)
        }))
        .into_any_element()
}

/// A track with a fill to `used`, in the colour of its pressure.
fn gauge_bar(used: f32, theme: &Theme) -> AnyElement {
    div()
        .flex_none()
        .w(px(GAUGE_WIDTH))
        .h(px(GAUGE_HEIGHT))
        .rounded_full()
        .bg(theme.muted_foreground.opacity(0.18))
        .child(
            div()
                .h_full()
                .w(gpui_kit::relative(used.clamp(0., 1.)))
                .rounded_full()
                .bg(Pressure::of(used).ink(theme)),
        )
        .into_any_element()
}

/// The processor and the memory, each as its own cluster.
fn load_clusters(parts: &StatusBar, load: &SystemLoad, theme: &Theme) -> (AnyElement, AnyElement) {
    let cpu = cluster(parts.part("cpu"), "status-cpu", load.cpu_tooltip(), theme)
        .child("CPU")
        .child(spark(&load.cpu_history, theme))
        .child(
            div()
                .min_w(px(28.))
                .text_color(Pressure::of(load.cpu).ink(theme))
                .child(load.cpu_words()),
        );
    let memory = cluster(parts.part("memory"), "status-memory", load.memory_tooltip(), theme)
        .child("RAM")
        .child(gauge_bar(load.memory_fraction(), theme))
        .child(load.memory_words());
    (cpu.into_any_element(), memory.into_any_element())
}

/// The sessions that wait on the reader. How many work is not shown: only what the reader owes the agents.
fn work_cluster(work: Work, theme: &Theme) -> Option<AnyElement> {
    if work.needs_you == 0 {
        return None;
    }
    Some(
        div()
            .debug_selector(|| "status-work".into())
            .flex()
            .flex_none()
            .items_center()
            .gap(px(8.))
            .child(div().text_color(theme.warning).child(work.needs_you_words()))
            .into_any_element(),
    )
}

/// The windows a chip shows, at most two: the short one and the long one (`5h` and `7d`) when the provider has them, else its
/// first two.
fn shown_windows(provider: &ProviderGauge) -> Vec<&crate::status_bar::Gauge> {
    let named: Vec<&crate::status_bar::Gauge> =
        ["5h", "7d"].iter().filter_map(|label| provider.gauges.iter().find(|g| g.label.as_ref() == *label)).collect();
    if named.is_empty() { provider.gauges.iter().take(2).collect() } else { named }
}

/// A provider as a compact chip: its mark, then each window as a label, a short track and its percent.
fn provider_chip(bar: &StatusBar, provider: &ProviderGauge, theme: &Theme) -> AnyElement {
    let debug = format!("status-provider-{}", provider.name);
    let id = ElementId::from((bar.id.clone(), format!("provider-{}", provider.name)));
    let dim = if provider.state == GaugeState::Live { 1. } else { 0.55 };
    let windows = if matches!(provider.state, GaugeState::Unavailable(_)) { Vec::new() } else { shown_windows(provider) };
    div()
        .id(id)
        .debug_selector(move || debug.clone())
        .flex()
        .flex_none()
        .items_center()
        .gap(px(8.))
        .opacity(dim)
        .text_color(theme.muted_foreground)
        .tooltip(Tooltip::text(provider.tooltip()))
        .child(lead_icon(&provider.name, provider.lead.clone(), MARK, theme))
        .children(windows.iter().map(|gauge| {
            let selector = format!("status-window-{}-{}", provider.name, gauge.label);
            div()
                .debug_selector(move || selector.clone())
                .flex()
                .flex_none()
                .items_center()
                .gap(px(5.))
                .child(div().child(gauge.label.clone()))
                .child(mini_gauge(gauge.fraction(), theme))
                .child(div().text_color(gauge.pressure().ink(theme)).child(gauge.percent()))
        }))
        .children(windows.is_empty().then(|| div().child("–")))
        .into_any_element()
}

/// A short track with a fill to `used`, for the chips.
fn mini_gauge(used: f32, theme: &Theme) -> AnyElement {
    div()
        .flex_none()
        .w(px(MINI_WIDTH))
        .h(px(GAUGE_HEIGHT))
        .rounded_full()
        .bg(theme.muted_foreground.opacity(0.18))
        .child(div().h_full().w(gpui_kit::relative(used.clamp(0., 1.))).rounded_full().bg(Pressure::of(used).ink(theme)))
        .into_any_element()
}

/// The app's version: a small mono label that is cut, not spilled, when the card is narrow.
fn version_label(bar: &StatusBar, version: SharedString, window: &mut Window, cx: &mut App, theme: &Theme) -> AnyElement {
    let label = div()
        .id(bar.part("version"))
        .debug_selector(|| "status-version".into())
        .flex_initial()
        .min_w_0()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .px(px(6.))
        .py(px(2.))
        .rounded(radius::md())
        .font_family(MONO_FONT_FAMILY)
        .text_color(theme.muted_foreground)
        .tooltip(Tooltip::text("What is new in this version"))
        .child(version);
    press_target(label, &focus_of(bar.part("version-focus"), window, cx), bar.on_version.clone(), window, theme)
}

/// The focus handle of a part, kept across frames.
fn focus_of(id: ElementId, window: &mut Window, cx: &mut App) -> FocusHandle {
    window.use_keyed_state(id, cx, |_, cx| cx.focus_handle()).read(cx).clone()
}

/// The provider chips as one target: a press on any of them opens the usage.
fn usage_target(bar: &StatusBar, chips: Vec<AnyElement>, window: &mut Window, cx: &mut App, theme: &Theme) -> AnyElement {
    let group = div()
        .id(bar.part("usage"))
        .debug_selector(|| "status-usage".into())
        .flex()
        .flex_none()
        .items_center()
        .gap(px(12.))
        .px(px(6.))
        .py(px(2.))
        .rounded(radius::md())
        .children(chips);
    press_target(group, &focus_of(bar.part("usage-focus"), window, cx), bar.on_usage.clone(), window, theme)
}

impl RenderOnce for StatusBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let (cpu, memory) = match self.load.as_ref().map(|load| load_clusters(&self, load, &theme)) {
            Some((cpu, memory)) => (Some(cpu), Some(memory)),
            None => (None, None),
        };
        let work = work_cluster(self.work, &theme);
        let chips: Vec<AnyElement> = self.providers.iter().map(|provider| provider_chip(&self, provider, &theme)).collect();
        let usage = (!chips.is_empty()).then(|| usage_target(&self, chips, window, cx, &theme));
        let version = self.version.clone().map(|version| version_label(&self, version, window, cx, &theme));
        let (lead, tail) = (self.lead, self.tail.filter(|_| self.load.is_some()));
        let mut cards: Vec<AnyElement> = Vec::new();
        let mut middle: Vec<AnyElement> = Vec::new();
        // The version has the sidebar's card to itself; with no column to stand under, it leads the middle card.
        match lead {
            Some(width) => cards.push(card(version.into_iter().collect(), "status-card-version", Some(width), &theme)),
            None => middle.extend(version),
        }
        middle.extend(work);
        middle.extend(usage);
        let machine: Vec<AnyElement> = cpu.into_iter().chain(memory).collect();
        match tail {
            Some(width) => {
                cards.push(card(middle, "status-card-main", None, &theme));
                cards.push(card(machine, "status-card-load", Some(width), &theme));
            }
            None => {
                middle.push(div().flex_1().into_any_element());
                middle.extend(machine);
                cards.push(card(middle, "status-card-main", None, &theme));
            }
        }
        div()
            .id(self.id.clone())
            .debug_selector(|| "status-bar".into())
            .flex()
            .flex_none()
            .items_center()
            .gap(px(GAP))
            .h(px(HEIGHT))
            .text_size(TextSize::Xs.font_size())
            .text_color(theme.muted_foreground)
            .children(cards)
    }
}
