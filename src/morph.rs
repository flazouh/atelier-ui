//! Swaps one child for another with the Claude app's text morph (`Mr` in `cb6930dae-j25SuahN.js`): the
//! old child rises out 3px and fades while the new one rises in from 3px below, both over 180ms on
//! `cubic-bezier(0.2, 0, 0, 1)`. Like Motion's `mode: "popLayout"`, the old child leaves the layout at
//! once, so both cross at the same time. Under Reduce Motion the new child shows at once.
//!
//! The child is a function, not an element, because GPUI elements live for one frame and the old
//! child must keep drawing after its key has changed.
//!
//! The exit (fading-out) and enter (rising-in) child each run on their own [`Channel`]. A key change
//! while settled starts both fresh, but a key change while a morph is already running leaves the
//! exit channel alone: the child that is fading out keeps its own value and never jumps. Only the
//! child that was rising in gets discarded and replaced; the truly new child restarts the enter
//! channel from nothing.

mod helpers;
mod structs;
mod types;

pub use helpers::frame;
pub use structs::{Morph, MorphFrame};

#[cfg(test)]
use helpers::{morph_curve, on_key_change};

#[cfg(test)]
use crate::motion::{Channel, MORPH_RISE};

#[cfg(test)]
mod tests;
