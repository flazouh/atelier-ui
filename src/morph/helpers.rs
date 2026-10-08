use std::time::Instant;

use super::structs::MorphFrame;
use crate::motion::{Channel, Curve, MORPH_RISE, duration, ease};

pub fn frame(p: f32) -> MorphFrame {
    MorphFrame {
        old_opacity: 1. - p,
        old_y: -MORPH_RISE * p,
        new_opacity: p,
        new_y: MORPH_RISE * (1. - p),
    }
}

pub(super) fn morph_curve() -> Curve {
    Curve::Ease(duration::MORPH.as_secs_f32(), ease::MORPH)
}

/// What a key change does to the running channels, kept as a pure function so the no-jump behavior
/// is testable without GPUI. `exit` is the channel already fading a previous child, or `None` when
/// the morph is settled.
///
/// - Settled (`exit` is `None`): the child that was current becomes the new exit, fading from full
///   opacity, and a fresh enter channel rises the new child in.
/// - Running (`exit` is `Some`): the exit channel is returned untouched, so the child fading out
///   keeps its exact value and never jumps. Only the enter channel restarts, from nothing.
pub(super) fn on_key_change(
    exit: Option<Channel>,
    reduce_motion: bool,
    now: Instant,
) -> (Channel, Channel) {
    let exit = exit.unwrap_or_else(|| {
        let mut c = Channel::new(0.);
        c.animate_at(1., morph_curve(), 0., reduce_motion, now);
        c
    });
    let mut enter = Channel::new(0.);
    enter.animate_at(1., morph_curve(), 0., reduce_motion, now);
    (exit, enter)
}
