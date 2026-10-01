//! New chat items arriving: one fade and rise for every block (a turn, a tool call, a plan, a diff, an
//! approval). An item fades in from 0 while rising from [`ENTER_RISE`] px below, over
//! [`duration::ENTER`] on [`ease::MORPH`], the Claude app's segment morph at a chat block's scale. Under
//! Reduce Motion it only fades, over [`duration::ENTER_REDUCED`].
//!
//! [`Entrance`] plays once per id. [`EntranceList`] decides which of its items are new: nothing on its
//! first paint (a loaded session), then every id it has not shown before, staggered by
//! [`STAGGER_STEP`] when several arrive in the same frame.

mod helpers;
mod structs;

pub use helpers::{curve, frame, stagger_delay};
pub use structs::{Arrivals, Entrance, EntranceFrame, EntranceList};

#[cfg(test)]
use gpui_kit::ElementId;
#[cfg(test)]
use crate::motion::{Curve, ENTER_RISE, STAGGER_CAP, STAGGER_STEP, duration, ease};

#[cfg(test)]
mod tests;
