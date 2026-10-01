use gpui_kit::{Styled, div};

use crate::scale::px;
use crate::{
    theme::{Theme},
};
use super::types::{MessageBubbleGroupSpacing, MessageBubbleVariant};

pub(super) fn surface_fill(variant: MessageBubbleVariant, theme: &Theme) -> Option<gpui_kit::Hsla> {
    match variant {
        MessageBubbleVariant::Solid => Some(theme.foreground),
        MessageBubbleVariant::Soft | MessageBubbleVariant::Tint => Some(theme.card),
        MessageBubbleVariant::Borderless => Some(theme.card_strong),
        MessageBubbleVariant::Danger => Some(theme.danger.opacity(0.1)),
        MessageBubbleVariant::Ghost => None,
    }
}

pub(super) fn content_color(variant: MessageBubbleVariant, theme: &Theme) -> gpui_kit::Hsla {
    match variant {
        MessageBubbleVariant::Solid => theme.background,
        MessageBubbleVariant::Danger => theme.danger,
        _ => theme.foreground,
    }
}

/// Stacks a speaker's bubbles into one grouped column.
pub fn message_bubble_group(spacing: MessageBubbleGroupSpacing) -> gpui_kit::Div {
    let gap = match spacing {
        MessageBubbleGroupSpacing::Compact => px(6.),
        MessageBubbleGroupSpacing::Default => px(12.),
    };
    div().flex().flex_col().w_full().gap(gap)
}
