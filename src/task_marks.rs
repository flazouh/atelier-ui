//! Marks for a task's status and priority, in Linear's model and drawn as vectors of our own in the
//! theme's colours: a dotted ring for Backlog, a ring for Todo, a ring with a pie that grows for In
//! Progress and In Review, a filled disc with a check for Done, a ring with a cross for Canceled; three bars
//! for a priority, a square with an exclamation mark for Urgent, and three dashes for none.

mod helpers;
mod structs;

pub use helpers::{label_color, label_tone_color, status_color};
pub use structs::{PriorityMark, TaskStatusMark};

#[cfg(test)]
use helpers::{pie, rect};

#[cfg(test)]
mod tests;
