use std::{collections::HashMap, sync::Arc, time::Instant};

use gpui_kit::{
    App, Div, ElementId, FontWeight, InteractiveElement, ParentElement, Stateful,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use super::structs::Warmth;
use super::types::{OPEN_DELAY, PILL_HEIGHT, PILL_TITLE_WIDTH, PrOpenHandler, SCHEME, WARM_FOR};
use crate::scale::px;
use crate::{
    focus::PressStop,
    icon::Icon,
    pr::PrChipData,
    pr_glance::size_text,
    pr_refs::pr_refs,
    theme::{Theme, mix, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

/// Rewrites every `#N` that `resolve` knows into a Markdown link to `atelier-pr:N`, and returns the data
/// for each. Numbers it does not know stay plain text.
pub fn link_prs(
    markdown: &str,
    resolve: impl Fn(u64) -> Option<PrChipData>,
) -> (String, HashMap<u64, PrChipData>) {
    let mut chips = HashMap::new();
    let mut out = String::with_capacity(markdown.len());
    let mut at = 0;
    for (range, number) in pr_refs(markdown) {
        let known = chips.contains_key(&number)
            || resolve(number).map(|pr| chips.insert(number, pr)).is_some();
        if known {
            out.push_str(&markdown[at..range.start]);
            out.push_str(&format!("[#{number}]({SCHEME}{number})"));
            at = range.end;
        }
    }
    out.push_str(&markdown[at..]);
    (out, chips)
}

/// The number in a link that [`link_prs`] wrote.
pub fn chip_number(url: &str) -> Option<u64> {
    url.strip_prefix(SCHEME)?.parse().ok()
}

/// Whether a card opens at once: one is open, or one closed a moment ago.
pub(super) fn warm(warmth: Option<&Warmth>, now: Instant) -> bool {
    warmth.is_some_and(|w| {
        w.open
            || w.changed
                .is_some_and(|at| now.duration_since(at) < WARM_FOR)
    })
}

pub(super) fn open_delay(cx: &App) -> std::time::Duration {
    if warm(cx.try_global::<Warmth>(), Instant::now()) {
        std::time::Duration::ZERO
    } else {
        OPEN_DELAY
    }
}

/// The pill: the state's mark, `#3344`, the title cut to fit, and the size once known.
pub(super) fn pill(
    id: &ElementId,
    pr: &PrChipData,
    on_open: Option<PrOpenHandler>,
    theme: &Theme,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div> {
    let child = |name: &'static str| ElementId::NamedChild(Arc::new(id.clone()), name.into());
    let open_pr = pr.clone();
    div()
        .id(child("pill"))
        .flex()
        .items_center()
        .gap(px(5.))
        .h(px(PILL_HEIGHT))
        .px(px(6.))
        .rounded(radius::md())
        .bg(theme.card_strong)
        .hover(|s| s.bg(mix(theme.card_strong, theme.foreground, 0.06)))
        .cursor_pointer()
        .text_size(TextSize::Xs.font_size())
        .line_height(px(PILL_HEIGHT))
        .when_some(on_open, |d, open| {
            d.press_stop((id.clone(), "chip-focus"), radius::md(), window, cx)
                .on_click(move |_, window, cx| open(&open_pr, window, cx))
        })
        .child(
            Icon::new(pr.state.icon())
                .size(px(12.))
                .color(pr.state.color(theme)),
        )
        .child(
            div()
                .flex_none()
                .font_family(MONO_FONT_FAMILY)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground.opacity(0.9))
                .child(pr.label()),
        )
        .child(
            div()
                .min_w_0()
                .max_w(px(PILL_TITLE_WIDTH))
                .truncate()
                .text_color(theme.foreground.opacity(0.75))
                .child(pr.short_title().to_string()),
        )
        .when_some(pr.facts.as_ref(), |d, facts| {
            d.child(size_text(facts.added, facts.removed, theme))
        })
}
