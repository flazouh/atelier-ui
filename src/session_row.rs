//! One session under its project in the sidebar: the agent's mark, the title, and how it stands. The
//! status is data; the row has no idea what an agent is.
//!
//! - Working: the agent's mark animates and the title is ink.
//! - Needs you: the warning tone and the words ("Needs approval").
//! - Finished and not yet seen: a small amber dot, the title in ink. Its changes are ready to look at.
//! - Seen and idle: the still mark, a muted title, the time since it last did anything.
//! - Failed: the danger tone and "Stopped: <reason>".
//!
//! A change of status rolls the old mark up out of its box and the new one up into it ([`crate::roll`]); with
//! Reduce Motion the mark changes at once.

mod helpers;
mod structs;
mod types;

pub use helpers::{agent_icon, status_mark, trailing};
pub use structs::SessionRow;
pub use types::{MARK_BOX, ROW_HEIGHT};

#[cfg(test)]
mod tests;
