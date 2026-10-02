use std::f32::consts::FRAC_PI_2;

/// How long one turn takes.
pub const TURN_MILLIS: u128 = 1000;

/// How long the reduced-motion pulse takes, and its lowest opacity.
pub const PULSE_MILLIS: u128 = 1400;

pub const PULSE_LOW: f32 = 0.4;

/// How strong the ring under the arc is.
pub const RING_ALPHA: f32 = 0.2;

/// The arc's length: a quarter of the ring.
pub(super) const SWEEP: f32 = FRAC_PI_2;

/// The steps a ring is drawn in.
pub(super) const STEPS: usize = 48;
