use gpui_kit::{
    AnyElement, App, FontWeight, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder,
};

use super::{
    super::{enums::UsageRange, structs::UsageDashboard},
    key::key,
};
use crate::{
    focus::PressStop,
    scale::px,
    theme::{Theme, radius},
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

/// The title, and the range as a segmented control.
pub(super) fn header(d: &UsageDashboard, theme: &Theme, window: &mut Window, cx: &mut App) -> AnyElement {
    let segments = UsageRange::ALL.map(|range| {
        let on = d.range == range;
        let handler = d.on_range.clone();
        let selector = format!("usage-range-{}", range.days());
        div()
            .id(key(&d.id, format!("range-{}", range.days())))
            .debug_selector(move || selector.clone())
            .px(px(13.))
            .py(px(6.))
            .rounded(radius::lg())
            .text_size(px(13.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(if on { theme.foreground } else { theme.muted_foreground })
            .when(on, |s| s.bg(theme.card_strong))
            .cursor_pointer()
            .press_stop(key(&d.id, format!("range-press-{}", range.days())), radius::lg(), window, cx)
            .on_click(move |_, window, cx| {
                if let Some(handler) = &handler {
                    handler(range, window, cx);
                }
            })
            .child(range.label())
    });
    div()
        .flex()
        .items_center()
        .justify_between()
        .h(px(44.))
        .child(div().text_size(px(20.)).font_weight(FontWeight::SEMIBOLD).child("Usage"))
        .child(div().flex().p(px(3.)).rounded(radius::xl()).bg(theme.card).children(segments))
        .into_any_element()
}
