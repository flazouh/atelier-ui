use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement,
    Styled, Window, div,
};

use crate::{
    menu::lead_icon,
    panel_layout::GAP,
    scale::px,
    theme::{ActiveTheme, Theme, radius},
    tooltip::Tooltip,
    typography::TextSize,
};

use super::super::{
    consts::{GAUGE_HEIGHT, GAUGE_WIDTH, HEIGHT, MARK, SPARK_BARS, SPARK_GAP, SPARK_HEIGHT, SPARK_WIDTH},
    enums::{GaugeState, Pressure},
    structs::{ProviderGauge, StatusBar, SystemLoad, Work},
};

impl StatusBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            load: None,
            work: Work::default(),
            providers: Vec::new(),
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

    fn part(&self, name: &'static str) -> ElementId {
        ElementId::from((self.id.clone(), name))
    }
}

/// What the bar shows as one card, as a panel is one: the card tone, the panels' corners, and a height that leaves the panels'
/// gap above and below it.
fn card(content: AnyElement, theme: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_none()
        .items_center()
        .h(px(HEIGHT - 2. * GAP))
        .px(px(10.))
        .rounded(radius::lg())
        .bg(theme.card)
        .child(content)
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

fn load_clusters(parts: &StatusBar, load: &SystemLoad, theme: &Theme) -> Vec<AnyElement> {
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
    vec![card(cpu.into_any_element(), theme), card(memory.into_any_element(), theme)]
}

fn work_cluster(work: Work, theme: &Theme) -> Option<AnyElement> {
    if work.is_idle() {
        return None;
    }
    Some(card(
        div()
            .debug_selector(|| "status-work".into())
            .flex()
            .flex_none()
            .items_center()
            .gap(px(8.))
            .children((work.working > 0).then(|| div().text_color(theme.accent).child(work.working_words())))
            .children((work.needs_you > 0).then(|| div().text_color(theme.warning).child(work.needs_you_words())))
            .into_any_element(),
        theme,
    ))
}

fn provider_chip(bar: &StatusBar, provider: &ProviderGauge, theme: &Theme) -> AnyElement {
    let debug = format!("status-provider-{}", provider.name);
    let id = ElementId::from((bar.id.clone(), format!("provider-{}", provider.name)));
    let dim = if provider.state == GaugeState::Live { 1. } else { 0.55 };
    let tightest = provider
        .tightest()
        .filter(|_| !matches!(provider.state, GaugeState::Unavailable(_)));
    let chip = div()
        .id(id)
        .debug_selector(move || debug.clone())
        .flex()
        .flex_none()
        .items_center()
        .gap(px(6.))
        .opacity(dim)
        .text_color(theme.muted_foreground)
        .tooltip(Tooltip::text(provider.tooltip()))
        .child(lead_icon(&provider.name, provider.lead.clone(), MARK, theme))
        .children(tightest.map(|gauge| gauge_bar(gauge.fraction(), theme)))
        .children(tightest.map(|gauge| {
            div()
                .text_color(provider.pressure().ink(theme))
                .child(format!("{} {}", gauge.label, gauge.percent()))
        }))
        .children(tightest.is_none().then(|| div().child("–")));
    card(chip.into_any_element(), theme)
}

impl RenderOnce for StatusBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let load = self
            .load
            .as_ref()
            .map(|load| load_clusters(&self, load, &theme))
            .unwrap_or_default();
        let providers: Vec<AnyElement> = self
            .providers
            .iter()
            .map(|provider| provider_chip(&self, provider, &theme))
            .collect();
        div()
            .id(self.id.clone())
            .debug_selector(|| "status-bar".into())
            .flex()
            .flex_none()
            .items_center()
            // Cards on the panels' gap. Its owner sets where it starts and ends, as it does for the panes.
            .gap(px(GAP))
            .h(px(HEIGHT))
            .text_size(TextSize::Xs.font_size())
            .text_color(theme.muted_foreground)
            .children(load)
            .children(work_cluster(self.work, &theme))
            .child(div().flex_1())
            .children(providers)
    }
}
