// These metrics are deliberate: they are the first mockup's (ring in a pill), not the DS `Button` Sm ones.
// All are in design pixels; the component scales them through `crate::scale::px`.
/// The button's height; its corner is half of it, a pill.
pub const HEIGHT: f32 = 28.;
pub const PAD_LEFT: f32 = 10.;
pub const PAD_RIGHT: f32 = 13.;
/// Between the ring or icon and the label.
pub const GAP: f32 = 8.;
pub const TEXT: f32 = 13.;
/// The ring's box and its stroke.
pub const RING: f32 = 16.;
pub const RING_STROKE: f32 = 2.;
pub const ICON: f32 = 14.;
/// The ink in the wash behind a button that cannot be pressed, and in the ring's track.
pub(super) const WASH: f32 = 0.07;
pub(super) const TRACK: f32 = 0.15;
/// How long the quarter arc of Restarting is, as a fraction of the ring.
pub(super) const SPIN_SWEEP: f32 = 0.28;
/// Steps in a full ring; a shorter arc takes a share of them.
pub(super) const RING_STEPS: usize = 48;
/// How fast the ring follows a new fraction (a critical spring: no overshoot).
pub(super) const FOLLOW_OMEGA: f32 = 18.;
