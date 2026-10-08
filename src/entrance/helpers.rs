use std::time::Duration;

use crate::motion::{Curve, ENTER_RISE, STAGGER_CAP, STAGGER_STEP, duration, ease};
use super::structs::EntranceFrame;

/// When the item at `index` of `count` items arriving together starts: [`STAGGER_STEP`] apart, but
/// never so late that it settles after [`STAGGER_CAP`].
pub fn stagger_delay(index: usize, count: usize) -> Duration {
    let index = index.min(count.saturating_sub(1)) as u32;
    (STAGGER_STEP * index).min(STAGGER_CAP - duration::ENTER)
}

pub fn frame(p: f32, reduce_motion: bool) -> EntranceFrame {
    EntranceFrame { opacity: p, y: if reduce_motion { 0. } else { ENTER_RISE * (1. - p) } }
}

pub fn curve(reduce_motion: bool) -> Curve {
    let length = if reduce_motion { duration::ENTER_REDUCED } else { duration::ENTER };
    Curve::Ease(length.as_secs_f32(), ease::MORPH)
}
