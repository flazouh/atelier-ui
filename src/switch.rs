//! Switch: beui.dev's Switch (`components/motion/switch.tsx`). A 48 x 28 track with a 20px thumb in it.
//!
//! - Track: `rounded-full`, 4px of padding. On, it is the primary; off, the muted ink at 60%. The colour
//!   changes over 200ms.
//! - Thumb: the page's colour with a `shadow-md`. It travels the 20px on a heavy spring (`{ stiffness: 800,
//!   damping: 80, mass: 4 }`): a damping ratio of 0.707, so it arrives with one overshoot of 4%.
//! - Press: with the pointer down the thumb squeezes to 90% and stretches 4px toward the side it will go
//!   from. A disabled switch (60%) shakes its thumb (`x: 0, -2, 2, -1, 0`, 600ms after 200ms) when pressed.
//! - Focus from the keyboard: a 2px ring with a 2px gap of the page between it and the track.
//!
//! The label, if any, follows at 12px, in the foreground at 14px, and a click on it also toggles. A key
//! that toggles the switch is shown once, after the label ([`Switch::cap`]). Under Reduce Motion the
//! thumb jumps and nothing squeezes.

mod helpers;
mod structs;
mod types;

pub use helpers::{shake_offset, thumb_span, track_fill};
pub use structs::Switch;
pub use types::{HEIGHT, PAD, SHAKE, SQUEEZE, STRETCH, THUMB, THUMB_SPRING, TRAVEL, WIDTH};

#[cfg(test)]
use helpers::span;
#[cfg(test)]
use structs::Dims;

#[cfg(test)]
use crate::motion::Animated;

#[cfg(test)]
mod tests;
