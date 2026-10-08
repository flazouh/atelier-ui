use gpui_kit::{FontWeight, Hsla, IntoElement, ParentElement, Styled, div};

use super::structs::{Geometry, NotificationItem};
use super::types::{CARD_PAD_Y, TrailingTone};
use crate::scale::px;
use crate::{icon::Icon, theme::Theme, typography::TextSize};

pub(super) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn trailing_color(theme: &Theme, tone: TrailingTone) -> Hsla {
    match tone {
        TrailingTone::Muted => theme.muted_foreground,
        TrailingTone::Warning => theme.warning,
        TrailingTone::Danger => theme.danger,
        TrailingTone::Success => theme.success,
    }
}

/// A card's words: the title, the small text at its right, and the description under them.
pub(super) fn card_words(item: &NotificationItem, theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .py(px(CARD_PAD_Y))
        .child(
            div()
                .flex()
                .items_start()
                .justify_between()
                .gap(px(12.))
                .child(
                    div()
                        .min_w_0()
                        .text_size(TextSize::Sm.font_size())
                        .line_height(px(19.25))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.foreground)
                        .child(item.title.clone()),
                )
                .children(item.trailing.clone().map(|t| {
                    let color = trailing_color(theme, t.tone);
                    div()
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap(px(4.))
                        .text_size(TextSize::Xs.font_size())
                        .line_height(px(16.))
                        .text_color(color)
                        .children(
                            t.icon
                                .map(|icon| Icon::new(icon).size(px(14.)).color(color)),
                        )
                        .child(t.text)
                })),
        )
        .children(item.description.clone().map(|d| {
            div()
                .text_size(TextSize::Xs.font_size())
                .line_height(px(19.5))
                .text_color(theme.muted_foreground)
                .child(d)
        }))
}

/// A card's top edge above the stack's bottom, to the bottom edge of the card above it: how far up from the bottom to put
/// the card's own bottom.
pub(super) fn stack_height_from(_: &Geometry, top: f32, height: f32) -> f32 {
    top - height
}
