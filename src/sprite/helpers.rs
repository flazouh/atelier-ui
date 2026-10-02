use std::{sync::OnceLock, time::Instant};

use super::structs::Strip;

/// The `(width, height)` in an SVG's `viewBox`. Some strips are exported a unit or two off an exact
/// `100 x frames*100`, so scaling the strip to a target width (as GPUI does, preserving the SVG's aspect
/// ratio) does not land each frame on a whole multiple of the requested size. Falls back to a square
/// 100-wide frame if `viewBox` is missing or malformed.
pub(crate) fn native_size(bytes: &[u8]) -> (f32, f32) {
    let fallback = (100., 100.);
    let text = std::str::from_utf8(bytes).unwrap_or_default();
    let Some(after) = text.split("viewBox=\"").nth(1) else { return fallback };
    let Some(value) = after.split('"').next() else { return fallback };
    let nums: Vec<f32> = value.split_whitespace().filter_map(|n| n.parse().ok()).collect();
    match nums.as_slice() {
        [_, _, w, h] => (*w, *h),
        _ => fallback,
    }
}

/// The pixel height of one frame when the strip is drawn `size` px wide, from its native
/// `(width, height)`. Equal to `size` when the viewBox is exactly `100 x frames*100`; a strip that is off
/// by a unit or two keeps its frames from drifting instead of assuming every frame is a perfect square.
pub(crate) fn frame_height(native: (f32, f32), frames: usize, size: f32) -> f32 {
    size * native.1 / native.0 / frames as f32
}

/// All sprites on a looping strip read from this same clock, so their frames stay in step with each
/// other no matter when each one mounted.
pub(super) fn shared_clock() -> Instant {
    static START: OnceLock<Instant> = OnceLock::new();
    *START.get_or_init(Instant::now)
}

/// Whether the animation should restart at frame 0: on a changed strip, or when play resumes after being
/// still. A still sprite never advances `start`, so without this a one-shot would jump to its last frame
/// and a loop would resume mid-cycle instead of starting over.
pub(crate) fn restarts(current: &Strip, current_still: bool, requested: &Strip) -> bool {
    current != requested || current_still
}
