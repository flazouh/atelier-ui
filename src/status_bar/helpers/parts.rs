//! What a card of the bar holds, one part at a time: the version, the work that waits, the usage of each provider, and the
//! machine. [`StatusBar`](super::super::StatusBar) puts them in its three cards; an app that composes its own cards
//! builds the same parts with the same ids, so a card looks and behaves the same either way.
//!
//! `bar` is the id of the bar the parts stand in: each part derives its own element ids from it.
use gpui_kit::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use crate::{
    menu::lead_icon,
    scale::px,
    theme::{ActiveTheme, Theme, radius},
    tooltip::Tooltip,
    typography::MONO_FONT_FAMILY,
};

use super::super::{
    consts::{
        GAUGE_HEIGHT, GAUGE_WIDTH, MARK, MINI_WIDTH, SPARK_BARS, SPARK_GAP, SPARK_HEIGHT,
        SPARK_WIDTH,
    },
    enums::{GaugeState, Pressure},
    structs::{Gauge, Press, ProviderGauge, SystemLoad, Work},
};
use super::press_target;

/// The id of the part `name` of the bar `bar`.
fn part(bar: &ElementId, name: &'static str) -> ElementId {
    ElementId::from((bar.clone(), name))
}

/// A caption and what it measures, with a hover.
fn cluster(
    id: ElementId,
    debug: &'static str,
    tooltip: String,
    theme: &Theme,
) -> gpui_kit::Stateful<gpui_kit::Div> {
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

/// The last samples of the processor, newest at the right, each in the colour of its own pressure and dimmer the older it is.
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
            div()
                .w(px(SPARK_WIDTH))
                .h(px(height))
                .rounded(px(1.))
                .bg(ink)
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

/// The processor and the memory, each as its own cluster, in that order.
pub fn load_parts(bar: &ElementId, load: &SystemLoad, cx: &App) -> Vec<AnyElement> {
    let theme = cx.theme();
    let cpu = cluster(part(bar, "cpu"), "status-cpu", load.cpu_tooltip(), theme)
        .child("CPU")
        .child(spark(&load.cpu_history, theme))
        .child(
            div()
                .min_w(px(28.))
                .text_color(Pressure::of(load.cpu).ink(theme))
                .child(load.cpu_words()),
        );
    let memory = cluster(
        part(bar, "memory"),
        "status-memory",
        load.memory_tooltip(),
        theme,
    )
    .child("RAM")
    .child(gauge_bar(load.memory_fraction(), theme))
    .child(load.memory_words());
    vec![cpu.into_any_element(), memory.into_any_element()]
}

/// The sessions that wait on the reader. How many work is not shown: only what the reader owes the agents.
pub fn work_part(work: Work, cx: &App) -> Option<AnyElement> {
    if work.needs_you == 0 {
        return None;
    }
    let theme = cx.theme();
    Some(
        div()
            .debug_selector(|| "status-work".into())
            .flex()
            .flex_none()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .text_color(theme.warning)
                    .child(work.needs_you_words()),
            )
            .into_any_element(),
    )
}

/// The windows a chip shows, at most two: the short one and the long one (`5h` and `7d`) when the provider has them, else its
/// first two.
fn shown_windows(provider: &ProviderGauge) -> Vec<&Gauge> {
    let named: Vec<&Gauge> = ["5h", "7d"]
        .iter()
        .filter_map(|label| provider.gauges.iter().find(|g| g.label.as_ref() == *label))
        .collect();
    if named.is_empty() {
        provider.gauges.iter().take(2).collect()
    } else {
        named
    }
}

/// A provider as a compact chip: its mark, then each window as a label, a short track and its percent.
fn provider_chip(bar: &ElementId, provider: &ProviderGauge, theme: &Theme) -> AnyElement {
    let debug = format!("status-provider-{}", provider.name);
    let id = ElementId::from((bar.clone(), format!("provider-{}", provider.name)));
    let dim = if provider.state == GaugeState::Live {
        1.
    } else {
        0.55
    };
    let windows = if matches!(provider.state, GaugeState::Unavailable(_)) {
        Vec::new()
    } else {
        shown_windows(provider)
    };
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
        .child(lead_icon(
            &provider.name,
            provider.lead.clone(),
            MARK,
            theme,
        ))
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
                .child(
                    div()
                        .text_color(gauge.pressure().ink(theme))
                        .child(gauge.percent()),
                )
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
        .child(
            div()
                .h_full()
                .w(gpui_kit::relative(used.clamp(0., 1.)))
                .rounded_full()
                .bg(Pressure::of(used).ink(theme)),
        )
        .into_any_element()
}

/// The version of the app: a small mono label that is cut, not spilled, when the card is narrow. A press runs `on_press`; with
/// none the label is a mark.
pub fn version_part(
    bar: &ElementId,
    version: SharedString,
    on_press: Option<Press>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let theme = cx.theme().clone();
    let label = div()
        .id(part(bar, "version"))
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
    press_target(
        label,
        &focus_of(part(bar, "version-focus"), window, cx),
        on_press,
        window,
        &theme,
    )
}

/// The focus handle of a part, kept across frames.
fn focus_of(id: ElementId, window: &mut Window, cx: &mut App) -> FocusHandle {
    window
        .use_keyed_state(id, cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone()
}

/// The provider chips as one target: a press on any of them runs `on_press` (the app opens the usage). With no reading to
/// show (none yet, or the providers did not answer) and a press to run, the usage still has its door, a label. With neither
/// there is nothing to draw.
pub fn usage_part(
    bar: &ElementId,
    providers: &[ProviderGauge],
    on_press: Option<Press>,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    let theme = cx.theme().clone();
    let mut chips: Vec<AnyElement> = providers
        .iter()
        .map(|provider| provider_chip(bar, provider, &theme))
        .collect();
    if chips.is_empty() && on_press.is_some() {
        chips.push(div().child("Usage").into_any_element());
    }
    if chips.is_empty() {
        return None;
    }
    let group = div()
        .id(part(bar, "usage"))
        .debug_selector(|| "status-usage".into())
        .flex()
        .flex_none()
        .items_center()
        .gap(px(12.))
        .px(px(6.))
        .py(px(2.))
        .rounded(radius::md())
        .children(chips);
    Some(press_target(
        group,
        &focus_of(part(bar, "usage-focus"), window, cx),
        on_press,
        window,
        &theme,
    ))
}
