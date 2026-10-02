//! AnimatedBadge: beui.dev's Animated Badge (`components/motion/animated-badge.tsx`). A pill with a status
//! icon and words. When the status or the words change, each rolls up out and the new one rolls up in
//! ([`crate::roll`]).
//!
//! - Sizes, in atelier's look (the old `Badge`: a pill with no border): Small is 20px tall with 8px across, 4px
//!   between, 11px words and a 12px icon; Medium is 24px, 10px, 6px, 12px words and a 14px icon.
//! - Statuses: Neutral (a ring), Info, Success (a check), Warning, Danger (a cross) and Loading (a
//!   turning ring). A tone is its colour for the words and the icon and a 14% wash for the fill, as the old
//!   `Badge` had it. Neutral is the muted words on the `card_strong` step.
//! - Loading pulses: a wash of the words' colour swells between 8% and 16% over 1.6s. With Reduce Motion it
//!   holds still and the ring does not turn.
//!
//! What gpui cannot draw is left out: the pulse's growth (it grows only in opacity), and the badge's spring
//! when its width changes (the width changes at once). Info and Loading use atelier's `info` tone, not the
//! primary, so they read on every primary the reader can choose.

mod helpers;
mod structs;
mod types;

pub use helpers::{colors, pulse_at};
pub use structs::AnimatedBadge;
pub use types::{BadgeSize, BadgeStatus, PULSE_MILLIS, PULSE_WASH};

#[cfg(test)]
mod tests;
