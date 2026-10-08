use gpui_kit::Hsla;

use super::types::{KEYFRAME_EASE, SHAKE, SHAKE_SECONDS};
use crate::{
    motion::keyframes,
    theme::{Theme, mix},
};

/// The field's sideways offset `t` seconds into a shake.
pub fn shake_offset(t: f32) -> f32 {
    let times: Vec<f32> = (0..SHAKE.len())
        .map(|i| i as f32 / (SHAKE.len() - 1) as f32)
        .collect();
    keyframes(&SHAKE, &times, SHAKE_SECONDS, KEYFRAME_EASE, t)
}

/// The idle border on `surface` (`border-border`).
pub fn edge(theme: &Theme, surface: Hsla) -> Hsla {
    mix(surface, theme.foreground, 0.12)
}

/// The field's fill on `surface`: the theme's `card_strong` step, so the field parts from its surface with no line.
pub fn fill(theme: &Theme, _surface: Hsla) -> Hsla {
    theme.card_strong
}
