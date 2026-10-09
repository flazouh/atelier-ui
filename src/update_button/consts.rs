// These are the DS `Button` Sm metrics (28 high, 10 padding, 6 gap, text 11 at normal weight; the ring is 14) and its default corner,
// as in the mockup Alex chose: the update button looks like our other buttons.
// All are in design pixels; the component scales them through `crate::scale::px`.
/// The button's height. Its corner is `radius::lg()`, as a `Button`'s.
pub const HEIGHT: f32 = 28.;
pub const PAD_LEFT: f32 = 10.;
pub const PAD_RIGHT: f32 = 10.;
/// Between the ring or icon and the label.
pub const GAP: f32 = 6.;
pub const TEXT: f32 = 11.;
/// The ring's box and its stroke.
pub const RING: f32 = 14.;
pub const RING_STROKE: f32 = 2.;
/// The ink in the wash behind a button that cannot be pressed, and in the ring's track.
pub(super) const WASH: f32 = 0.07;
pub(super) const TRACK: f32 = 0.15;
/// How long the quarter arc of Restarting is, as a fraction of the ring.
pub(super) const SPIN_SWEEP: f32 = 0.28;
/// Steps in a full ring; a shorter arc takes a share of them.
pub(super) const RING_STEPS: usize = 48;
/// How fast the ring follows a new fraction (a critical spring: no overshoot).
pub(super) const FOLLOW_OMEGA: f32 = 18.;
