use gpui_kit::{
    AnyElement, FontWeight, InteractiveElement, IntoElement, ParentElement, Styled, div,
    prelude::FluentBuilder,
};

use super::{
    super::structs::{UsageDashboard, UsageStat},
    gauge::gauge,
};
use crate::{
    scale::px,
    theme::{Theme, radius},
};

/// The row of figures under the title: each a tile with its label, its big value and unit, its meter and its note.
pub(super) fn stats(d: &UsageDashboard, theme: &Theme) -> AnyElement {
    div()
        .debug_selector(|| "usage-stats".into())
        .flex()
        .gap(px(10.))
        .children(
            d.stats
                .iter()
                .enumerate()
                .map(|(at, stat)| tile(at, stat, theme)),
        )
        .into_any_element()
}

fn tile(at: usize, stat: &UsageStat, theme: &Theme) -> AnyElement {
    div()
        .debug_selector(move || format!("usage-stat-{at}"))
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .px(px(16.))
        .py(px(14.))
        .rounded(radius::xl())
        .bg(theme.card_strong)
        .child(
            div()
                .text_size(px(12.5))
                .text_color(theme.muted_foreground)
                .truncate()
                .child(stat.label.clone()),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_baseline()
                .gap_x(px(6.))
                .mt(px(4.))
                .child(
                    div()
                        .text_size(px(24.))
                        .font_weight(FontWeight::MEDIUM)
                        .child(stat.value.clone()),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.muted_foreground)
                        .child(stat.unit.clone()),
                ),
        )
        .when_some(stat.used, |t, used| {
            t.child(div().mt(px(10.)).child(gauge(used, theme)))
        })
        .child(
            div()
                .mt(px(8.))
                .text_size(px(12.))
                .text_color(theme.muted_foreground)
                .truncate()
                .child(stat.note.clone()),
        )
        .into_any_element()
}
