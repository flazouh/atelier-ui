//! Spinner: the `spinner` variant of beui.dev's Loader (`components/motion/loader.tsx`). A ring drawn at
//! 20% with a quarter arc on it, round at both ends, turning once a second at a steady speed. The stroke is
//! 9% of the size and never under 2px. Under Reduce Motion nothing turns: the whole ring pulses from full
//! to 40% and back over 1.4s.
//!
//! The other sixteen variants of the Loader (dots, bars, dot-matrix, dither, the ascii sets, morph, comet,
//! scramble, metaballs, newton, helix, percent) are not built; atelier shows only work in progress.

mod helpers;
mod structs;
mod types;

pub use helpers::{arc, pulse_at, stroke_width, turn_at};
pub use structs::Spinner;
pub use types::{PULSE_LOW, PULSE_MILLIS, RING_ALPHA, TURN_MILLIS};

#[cfg(test)]
mod tests;
