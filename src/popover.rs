//! One popover for everything that opens over the page: the pickers, the menus, the finder, the cards.
//!
//! It is controlled: the owner keeps the `open` flag and clears it in [`Popover::on_close`]. What it owns
//! is the behaviour every popover shares, so no picker has to build it again:
//!
//! - A full-window backdrop under the panel takes every press. A click outside closes the popover and does
//!   nothing else: it does not press what is under it, start a drag or a selection, or reach a tooltip, a
//!   hover or a cursor. A click on the trigger, which the backdrop also covers, closes it too, so the
//!   trigger toggles.
//! - Escape closes it and focus returns to the trigger. Tab closes it and moves on from the trigger.
//! - One popover is open at a time: opening one closes the last.
//! - A switchable popover's trigger stays live while another switchable popover is open: a press on it
//!   closes the open one and opens its own, as a menu bar does ([`Popover::switchable`]).
//! - It closes when its trigger moves (a scroll), the window resizes or loses focus, or the view that shows
//!   it stops drawing it (another story, tab or pane).
//! - It sits above everything (the deferred priority is [`PRIORITY`]), opens above the trigger when there
//!   is no room below and more above ([`crate::placement`]), and slides into the window at the edges.
//!
//! The panel is the owner's: its fill, radius, shadow and motion. The owner measures the trigger with
//! [`crate::placement::measure`] and passes the bounds.

mod helpers;
mod structs;
mod types;

pub use helpers::placement;
pub use structs::Popover;
pub use types::{Align, CloseHandler, Hang, PRIORITY, Side};

#[cfg(test)]
use helpers::cover;
#[cfg(test)]
use structs::Frames;

#[cfg(test)]
use gpui_kit::Anchor;

#[cfg(test)]
mod tests;
