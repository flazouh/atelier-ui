use gpui_kit::{App, Entity, Hsla, IntoElement, ParentElement, Styled, div, transparent_black};

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    theme::{Theme, mix, radius},
};
use super::structs::{ButtonMotion, Metrics};
use super::types::ButtonVariant;

/// Changes the pointer state of a button and restarts its motion toward the new look.
pub(super) fn update_motion(motion: &Entity<ButtonMotion>, cx: &mut App, change: impl FnOnce(&mut ButtonMotion)) {
    let reduce = cx.reduce_motion();
    motion.update(cx, |m, cx| {
        change(m);
        m.retarget(reduce);
        cx.notify();
    });
}

/// Fill and text at hover progress `hover`. With a chip, the primary fill stays still, as on mem0: the
/// chip moves instead.
pub(crate) fn colors(variant: ButtonVariant, theme: &Theme, hover: f32, has_chip: bool) -> (Hsla, Hsla) {
    match variant {
        ButtonVariant::Primary => (
            if has_chip { theme.primary } else { mix(theme.primary, theme.primary_hover(), hover) },
            theme.primary_foreground,
        ),
        // Borderless: one step above a card, so it still shows when it sits on one.
        ButtonVariant::Secondary => (mix(theme.card_strong, theme.foreground, 0.05 * hover), theme.foreground),
        ButtonVariant::Ghost => (
            mix(transparent_black(), theme.muted_hover(), hover),
            mix(theme.muted_foreground, theme.foreground, hover),
        ),
        ButtonVariant::Tinted => (theme.foreground.opacity(0.09 + 0.05 * hover), theme.foreground),
        ButtonVariant::Invert => (mix(theme.foreground, theme.background, 0.1 * hover), theme.background),
    }
}

/// The chip: two copies of the icon, one leaving through the top while the other comes in from below.
pub(super) fn chip(icon: IconName, m: &Metrics, tint: f32, slide: f32, theme: &Theme) -> impl IntoElement {
    let size = m.height - m.chip_inset * 2.;
    let rest_top = (size - m.icon) / 2.;
    // mem0 moves its arrows 32px on a 22px chip.
    let travel = size * 32. / 22.;
    let shift = travel * slide;
    let arrow = |color: Hsla, top: f32| {
        div()
            .absolute()
            .left(px((size - m.icon) / 2.))
            .top(px(top))
            .child(Icon::new(icon).size(px(m.icon)).color(color))
    };
    div()
        .relative()
        .flex_none()
        .size(px(size))
        .rounded(radius::lg())
        .overflow_hidden()
        .bg(mix(theme.chip_rest, theme.chip_hover, tint))
        .child(arrow(theme.chip_arrow, rest_top - shift))
        .child(arrow(theme.chip_rest, rest_top + travel - shift))
}

/// A small round color mark.
pub fn dot(color: Hsla) -> impl IntoElement {
    div().relative().flex_none().size(px(6.)).rounded_full().bg(color)
}

/// The hover level a button eases toward: 1 on the pointer, and 1 while its picker is open.
pub(super) fn hover_target(hovered: bool, held: bool) -> f32 {
    if hovered || held { 1. } else { 0. }
}
