use std::time::Instant;

use crate::motion::{cubic_bezier, ease};

/// How far `started` is into a step of length `step`, eased, or `None` once the step is over.
pub(super) fn progress(started: Instant, now: Instant, step: std::time::Duration) -> Option<f32> {
    let t = now.saturating_duration_since(started).as_secs_f32() / step.as_secs_f32();
    (t < 1.).then(|| cubic_bezier(ease::MORPH, t))
}
