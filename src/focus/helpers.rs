use gpui_kit::{BoxShadow, Hsla, InteractiveElement, IntoElement, Pixels, Styled, div, point};

use crate::scale::px;
use crate::theme::{MARK_CONTRAST, Theme, contrast, mix};
use super::types::RING_WIDTH;

/// The ring's colour on `surface`: the ink, mixed toward `surface` no further than it must be for 3:1.
pub fn ring_color(theme: &Theme, surface: Hsla) -> Hsla {
    (1..=20)
        .map(|step| mix(surface, theme.foreground, step as f32 / 20.))
        .find(|ring| contrast(*ring, surface) >= MARK_CONTRAST)
        .unwrap_or(theme.foreground)
}

/// The ring as a shadow with no blur and a spread of [`RING_WIDTH`], drawn outside the control's box.
pub fn ring_shadow(theme: &Theme, surface: Hsla) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: ring_color(theme, surface),
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(RING_WIDTH),
        inset: false,
    }]
}

/// The ring for a row that has the keyboard cursor: the same 2px in the same colour, drawn inside the row's edge, so a
/// neighbour or a scroll box does not clip it. Put it as the last child of a `relative` row of the same `radius`.
pub fn row_ring(theme: &Theme, surface: Hsla, radius: Pixels) -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .rounded(radius)
        .border(px(RING_WIDTH))
        .border_color(ring_color(theme, surface))
        .debug_selector(|| "row-ring".into())
}
