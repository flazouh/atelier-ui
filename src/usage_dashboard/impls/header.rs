use gpui_kit::{
    AnyElement, App, FontWeight, InteractiveElement, IntoElement, ParentElement, Styled, Window,
    div, prelude::FluentBuilder,
};

use super::{
    super::{consts::DOT, enums::UsageRange, structs::UsageDashboard},
    key::key,
};
use crate::{
    badge::Badge,
    scale::px,
    tabs::{Tab, Tabs, TabsVariant},
    theme::Theme,
};

impl UsageRange {
    pub const ALL: [UsageRange; 3] = [Self::Week, Self::Fortnight, Self::Month];

    /// How many days it looks back.
    pub fn days(self) -> usize {
        match self {
            Self::Week => 7,
            Self::Fortnight => 14,
            Self::Month => 30,
        }
    }

    /// "14 days".
    pub fn label(self) -> String {
        format!("{} days", self.days())
    }
}

/// The title of what is chosen, with its provider and what it is, and the range as the Tabs Segment control.
pub(super) fn header(
    d: &UsageDashboard,
    theme: &Theme,
    _window: &mut Window,
    _cx: &mut App,
) -> AnyElement {
    let handler = d.on_range.clone();
    let tabs = Tabs::new(
        key(&d.id, "range".to_string()),
        TabsVariant::Segment,
        UsageRange::ALL.map(|range| {
            Tab::new(range.label()).debug_name(format!("usage-range-{}", range.days()))
        }),
        UsageRange::ALL.iter().position(|range| *range == d.range),
    )
    .on_select(move |at, window, cx| {
        if let (Some(handler), Some(range)) = (&handler, UsageRange::ALL.get(at)) {
            handler(*range, window, cx);
        }
    });
    let dot = d.dot.map_or(theme.foreground, |series| {
        theme.series(series.hue, series.shade)
    });
    div()
        .flex()
        .items_start()
        .justify_between()
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(3.))
                .min_w_0()
                .child(
                    div()
                        .debug_selector(|| "usage-title".into())
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(div().flex_none().size(px(DOT + 2.)).rounded_full().bg(dot))
                        .child(
                            div()
                                .text_size(px(24.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .truncate()
                                .child(d.title.clone()),
                        )
                        .when_some(d.provider.clone(), |row, provider| {
                            row.child(Badge::new(provider))
                        }),
                )
                .child(
                    div()
                        .ml(px(DOT + 12.))
                        .text_size(px(13.5))
                        .text_color(theme.muted_foreground)
                        .truncate()
                        .child(d.subtitle.clone()),
                ),
        )
        .child(tabs)
        .into_any_element()
}
