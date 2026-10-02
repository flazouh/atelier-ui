//! Amber bars for a voice: while it records, a row of rounded bars swells and ripples with the level of the
//! microphone, and settles to a line of small dots when it is quiet or off.
//!
//! SPIKE: the first look at a recording state for the prompt input. The level comes from the caller, 0 to 1.
//!
//! - The shape of the field comes from four drifting sine ribbons, each at its own speed and direction. Each bar
//!   reads the ribbons at its own spot, so the crests travel through the row instead of every bar bouncing alone.
//! - A window `sin(pi x)^1.6` tapers the row to dots at both ends, so the bars float in their space.
//! - The level is smoothed with a fast attack and a slow release, so a word lands at once and fades gently.
//! - The phase moves faster the louder the voice, which makes the waves feel pushed by it.
//! - Taller bars are also brighter, which gives the row depth without a second color.
//! - Reduce Motion: the bars hold still at their current size.

mod helpers;
mod structs;
mod types;

pub use helpers::{amber, amber_for, amplitude, bar, on_amber, smooth, window};
pub use structs::{Ribbon, VoiceWaves};
pub use types::{BAR_LIFT, FLOOR, MAX_BAR_WIDTH, MIN_BAR, RIBBONS};

#[cfg(test)]
mod tests;
