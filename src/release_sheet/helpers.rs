use gpui_kit::Hsla;
use crate::motion::{cubic_bezier, ease};
use super::consts::{DRIFT_SECONDS, DRIFT_WAKE_AT, DRIFT_WAKE_SECONDS, SPARE, ZOOM, ZOOM_SECONDS};

/// How far a piece that starts at `delay` seconds and takes `span` seconds has come at `elapsed` seconds: 0 to 1 on
/// `ease::OUT`.
pub fn reveal(elapsed: f32, delay: f32, span: f32) -> f32 {
    cubic_bezier(ease::OUT, ((elapsed - delay) / span).clamp(0., 1.))
}

/// How many px the picture still sticks out past its place at `elapsed` seconds: `ZOOM` at the start, none once it
/// has zoomed in.
pub fn zoom(elapsed: f32) -> f32 {
    ZOOM * (1. - reveal(elapsed, 0., ZOOM_SECONDS))
}

/// How far the picture has drifted from the middle at `elapsed` seconds, as (across, down) in px. Two slow sines of
/// different lengths never repeat in step. The reach stays under `SPARE`, so no border shows.
pub fn drift(elapsed: f32) -> (f32, f32) {
    let wake = reveal(elapsed, DRIFT_WAKE_AT, DRIFT_WAKE_SECONDS);
    let turn = std::f32::consts::TAU * elapsed;
    ((turn / DRIFT_SECONDS).sin() * SPARE * 0.7 * wake, (turn / (DRIFT_SECONDS * 1.4) + 1.).sin() * SPARE * 0.45 * wake)
}

/// `color` at `t` of its strength.
pub(super) fn fade(color: Hsla, t: f32) -> Hsla {
    Hsla { a: color.a * t, ..color }
}
