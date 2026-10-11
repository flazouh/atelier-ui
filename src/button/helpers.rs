use gpui_kit::{App, Background, Entity, Hsla, IntoElement, ParentElement, Styled, div, linear_color_stop, linear_gradient, transparent_black};

use crate::atelier_mark::Tile;

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    theme::{Theme, mix, radius},
};
use super::structs::{ButtonMotion, Metrics};
use super::types::ButtonVariant;

/// The padding at the side of a button: the size's when it has words, else just what centres the icon in a square as
/// tall as the button, so an icon alone in a text size (the add button of a group) is not a wide button.
pub(super) fn side_pad(m: &Metrics, words: bool) -> f32 {
    if words { m.pad_x } else { (m.height - m.icon) / 2. }
}

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
/// The fill of a button that was given one of its own (`Button::fill`), with its words in `ink`: under the pointer
/// it steps toward its words, as the invert button does.
pub(crate) fn own_fill(fill: Hsla, ink: Hsla, hover: f32) -> Hsla {
    mix(fill, ink, 0.1 * hover)
}

pub(crate) fn colors(variant: ButtonVariant, theme: &Theme, hover: f32, has_chip: bool) -> (Hsla, Hsla) {
    match variant {
        ButtonVariant::Primary => (
            if has_chip { theme.primary } else { mix(theme.primary, theme.primary_hover(), hover) },
            theme.primary_foreground,
        ),
        // Borderless: a wash of the ink, so it steps up from the page, a card or a box inside a card alike.
        ButtonVariant::Secondary => (
            theme.foreground.opacity(0.07 + 0.04 * hover),
            theme.foreground,
        ),
        ButtonVariant::Ghost => (
            mix(transparent_black(), theme.muted_hover(), hover),
            mix(theme.muted_foreground, theme.foreground, hover),
        ),
        ButtonVariant::Tinted => (theme.foreground.opacity(0.09 + 0.05 * hover), theme.foreground),
        ButtonVariant::Send => (mix(theme.send, theme.send_foreground, 0.17 * hover), theme.send_foreground),
        ButtonVariant::Stop => (mix(theme.stop, theme.send_foreground, 0.17 * hover), theme.send_foreground),
        ButtonVariant::Invert => (mix(theme.foreground, theme.background, 0.1 * hover), theme.background),
    }
}

/// How strong the chip is, as a wash of the button's words over its fill: quiet at rest, a little stronger under
/// the pointer.
pub(super) const CHIP_REST: f32 = 0.10;
pub(super) const CHIP_HOVER: f32 = 0.20;

/// The chip's fill on a button whose words are `ink`, `tint` of the way from rest to hovered.
pub(crate) fn chip_fill(ink: Hsla, tint: f32) -> Hsla {
    ink.opacity(CHIP_REST + (CHIP_HOVER - CHIP_REST) * tint.clamp(0., 1.))
}

/// How much of a tile the chip wears at rest ([`super::Button::chip_tile`]); under the pointer it wears it whole.
pub(super) const TILE_REST: f32 = 0.24;

/// How strong the chip's tile is, `tint` of the way from rest to hovered.
pub(crate) fn tile_strength(tint: f32) -> f32 {
    TILE_REST + (1. - TILE_REST) * tint.clamp(0., 1.)
}

/// The chip: two copies of the icon, one leaving through the top while the other comes in from below.
///
/// - With no tile it takes its colours from the button it is on: a wash of the words' colour, and the icon in
///   the words' colour.
/// - With a tile (the mark's) it wears the tile's colours, faint at rest and whole under the pointer. The icon
///   at rest is in the words' colour, and the one that comes in is in the tile's letter colour, as on the mark.
pub(super) fn chip(icon: IconName, m: &Metrics, tint: f32, slide: f32, ink: Hsla, tile: Option<Tile>) -> impl IntoElement {
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
    let fill = match tile {
        None => Background::from(chip_fill(ink, tint)),
        Some(tile) => {
            let strength = tile_strength(tint);
            linear_gradient(180., linear_color_stop(tile.top.opacity(strength), 0.), linear_color_stop(tile.bottom.opacity(strength), 1.))
        }
    };
    div()
        .relative()
        .flex_none()
        .size(px(size))
        .rounded(radius::lg())
        .overflow_hidden()
        .bg(fill)
        .child(arrow(ink, rest_top - shift))
        .child(arrow(tile.map_or(ink, |tile| tile.letter), rest_top + travel - shift))
}

/// A small round color mark.
pub fn dot(color: Hsla) -> impl IntoElement {
    div().relative().flex_none().size(px(6.)).rounded_full().bg(color)
}

/// The hover level a button eases toward: 1 on the pointer, and 1 while its picker is open.
pub(super) fn hover_target(hovered: bool, held: bool) -> f32 {
    if hovered || held { 1. } else { 0. }
}
