use crate::motion::{cubic_bezier, ease};

use super::consts::{
    ACTION_AT, ACTION_SECONDS, BREATH_PEAK, BREATH_SECONDS, INTRO_SECONDS, LINE_AT, LINE_SECONDS, MARK_AT,
    MARK_SECONDS, PICTURE_SECONDS, SWING, SWING_SECONDS, ZOOM_FROM, ZOOM_SECONDS,
};

/// Where every part of the page stands at one moment: 0 is hidden and low, 1 is at rest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Frame {
    pub picture: f32,
    /// The first picture's size, as a share of the page: a little over 1 as the page opens, then a slow swing.
    pub zoom: f32,
    /// The second picture's strength over the first.
    pub breath: f32,
    pub mark: f32,
    pub line: f32,
    pub action: f32,
}

/// The whole page at rest.
const REST: Frame = Frame { picture: 1., zoom: 1., breath: 0., mark: 1., line: 1., action: 1. };

/// How far a part that starts at `at` and takes `seconds` has come at `elapsed`, eased out.
fn progress(elapsed: f32, at: f32, seconds: f32) -> f32 {
    let t = (elapsed - at) / seconds;
    if t <= 0. {
        0.
    } else if t >= 1. {
        1.
    } else {
        cubic_bezier(ease::OUT, t)
    }
}

/// Half of one minus the cosine: a wave from 0 up to 1 and back, once per `period`, starting at 0.
fn wave(elapsed: f32, period: f32) -> f32 {
    (1. - (std::f32::consts::TAU * elapsed / period).cos()) / 2.
}

/// The page `elapsed` seconds after its first frame. Under Reduce Motion it is at rest from the start.
pub(super) fn frame(elapsed: f32, reduce: bool) -> Frame {
    if reduce {
        return REST;
    }
    Frame {
        picture: progress(elapsed, 0., PICTURE_SECONDS),
        zoom: ZOOM_FROM + (1. - ZOOM_FROM) * progress(elapsed, 0., ZOOM_SECONDS) + SWING * wave(elapsed, SWING_SECONDS),
        breath: BREATH_PEAK * wave(elapsed, BREATH_SECONDS),
        mark: progress(elapsed, MARK_AT, MARK_SECONDS),
        line: progress(elapsed, LINE_AT, LINE_SECONDS),
        action: progress(elapsed, ACTION_AT, ACTION_SECONDS),
    }
}

/// Whether the opening is over: after it, only the breath asks for frames.
pub(super) fn intro_done(elapsed: f32) -> bool {
    elapsed >= INTRO_SECONDS
}
