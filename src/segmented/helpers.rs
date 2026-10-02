use gpui_kit::Hsla;

use crate::theme::{Theme, mix};
use super::types::{PRESS_SCALE, SEGMENT_HEIGHT};

/// A segment's fill at choice progress `chosen` (0 to 1) and hover progress `hover`: the `Ghost` button's,
/// or the `Secondary` button's when chosen.
pub fn segment_fill(theme: &Theme, chosen: f32, hover: f32) -> Hsla {
    let ghost = crate::button::colors(crate::button::ButtonVariant::Ghost, theme, hover, false).0;
    let secondary =
        crate::button::colors(crate::button::ButtonVariant::Secondary, theme, hover, false).0;
    mix(ghost, secondary, chosen)
}

/// A segment's text: muted, then the foreground when chosen or hovered (`hover:text-foreground`).
pub fn segment_text(theme: &Theme, chosen: f32, hover: f32) -> Hsla {
    mix(theme.muted_foreground, theme.foreground, chosen.max(hover))
}

/// How far a pill pulls in from each side at press progress `press`: a share of its width across, pixels
/// down. At full press the pill is 95% of its size.
pub fn pill_inset(press: f32) -> (f32, f32) {
    let shrink = (1. - PRESS_SCALE) * press;
    (shrink / 2., SEGMENT_HEIGHT * shrink / 2.)
}
