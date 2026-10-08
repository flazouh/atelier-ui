use gpui_kit::SharedString;

use super::structs::Swatch;
use super::types::{DISC, DOT, RING_OUT};

/// The index the arrow keys reach from `from`: the next (or previous) swatch that can be chosen, wrapping
/// at the ends like a radio group. `None` when nothing else can be.
pub fn step(swatches: &[Swatch], from: usize, forward: bool) -> Option<usize> {
    let n = swatches.len();
    (1..n)
        .map(|k| {
            if forward {
                (from + k) % n
            } else {
                (from + n - k) % n
            }
        })
        .find(|&i| !swatches[i].disabled)
}

/// The swatch that Tab reaches: the chosen one, or the first that can be chosen.
pub fn tab_stop(swatches: &[Swatch], value: Option<&SharedString>) -> Option<usize> {
    value
        .and_then(|v| swatches.iter().position(|s| &s.value == v && !s.disabled))
        .or_else(|| swatches.iter().position(|s| !s.disabled))
}

/// The disc's, the dot's and the ring's sizes at a press scale.
pub fn sizes(scale: f32) -> (f32, f32, f32) {
    (DISC * scale, DOT * scale, (DISC + 2. * RING_OUT) * scale)
}
