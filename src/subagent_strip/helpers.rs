use std::time::Instant;

use crate::{
    motion::{Channel, Curve, Spring},
    };

pub(super) fn open(to: f32, reduce: bool, now: Instant, from: f32) -> Channel {
    let mut c = Channel::new(from);
    c.animate_at(to, Curve::Spring(Spring::LAYOUT), 0., reduce, now);
    c
}
