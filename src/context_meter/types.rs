/// How full the window is, as the ring's colour tells it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Room,
    Filling,
    Full,
}

/// The share of the window from which the ring turns amber.
pub const WARN_AT: f32 = 0.8;

/// The share of the window from which the ring turns red.
pub const FULL_AT: f32 = 0.95;

pub(super) const SIZE: f32 = 14.;

pub(super) const STROKE: f32 = 2.;

/// The steps a whole ring is drawn in.
pub(super) const STEPS: usize = 48;

/// The box round the ring, so it lines up with the toolbar's other 32px buttons.
pub(super) const SLOT: f32 = 32.;

pub(super) const THOUSAND: u64 = 1_000;

pub(super) const MILLION: u64 = 1_000_000;
