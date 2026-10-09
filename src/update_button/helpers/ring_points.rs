use std::f32::consts::TAU;

use crate::{
    spinner::{arc, turn_at},
    update_button::consts::{RING, RING_STEPS, RING_STROKE},
};

/// Points of the ring's arc in design pixels inside the ring's box: from `start` (radians from 12 o'clock) clockwise
/// over `fraction` of a turn. Nothing for no progress.
pub(in crate::update_button) fn ring_points(fraction: f32, start: f32) -> Vec<(f32, f32)> {
    let fraction = fraction.clamp(0., 1.);
    if fraction <= 0. {
        return Vec::new();
    }
    let steps = ((RING_STEPS as f32 * fraction).ceil() as usize).max(2);
    arc((RING / 2., RING / 2.), (RING - RING_STROKE) / 2., start, TAU * fraction, steps)
}

/// Where the turning arc starts, `millis` in; it stands at 12 o'clock under Reduce Motion.
pub(in crate::update_button) fn ring_start(millis: u128, reduce: bool) -> f32 {
    if reduce { 0. } else { turn_at(millis) }
}
