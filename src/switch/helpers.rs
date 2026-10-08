use gpui_kit::Hsla;

use super::structs::Dims;
use super::types::{EASE_IN_OUT, SHAKE, SHAKE_DELAY, SHAKE_SECONDS, STRETCH};
use crate::{
    motion::keyframes,
    theme::{Theme, mix},
};

/// The thumb's sideways offset `t` seconds after a press on a disabled switch.
pub fn shake_offset(t: f32) -> f32 {
    if t <= SHAKE_DELAY {
        return 0.;
    }
    let times: Vec<f32> = (0..SHAKE.len())
        .map(|i| i as f32 / (SHAKE.len() - 1) as f32)
        .collect();
    keyframes(&SHAKE, &times, SHAKE_SECONDS, EASE_IN_OUT, t - SHAKE_DELAY)
}

/// The thumb's left edge and width inside the track at travel `t` (0 off, 1 on) with the squeeze at
/// `press` (0 to 1) and the switch `on`. The stretch grows toward the side the thumb came from.
pub fn thumb_span(t: f32, press: f32, on: bool) -> (f32, f32) {
    span(Dims::STANDARD, t, press, on)
}

pub(super) fn span(dims: Dims, t: f32, press: f32, on: bool) -> (f32, f32) {
    let stretch = STRETCH * press;
    let left = dims.pad + dims.travel() * t - if on { stretch } else { 0. };
    (left, dims.thumb + stretch)
}

/// The track's fill: the primary when on, the muted ink at 60% over `page` when off.
pub fn track_fill(theme: &Theme, page: Hsla, on: f32) -> Hsla {
    mix(mix(page, theme.muted_foreground, 0.6), theme.primary, on)
}
