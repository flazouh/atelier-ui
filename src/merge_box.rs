//! The merge box in a pull request's rail, as GitQuiet's merge card: the standing in one line ("Ready
//! to merge", "2 checks still running", "Blocked: changes asked by Ada"), each reason as a row with
//! what to do about it, the squash commit's title and message for a squash (editable, filled from the
//! pull request's title and body), and the [`MergeButton`]. After a merge it shows the result: merged,
//! the branch deleted or a "Delete branch" button, and "Revert".
//!
//! The box asks nothing itself: a press or a change of choice is an event, and the owner writes to
//! the forge and hands the new facts back ([`MergeBox::set_facts`], [`MergeBox::merged`]).

mod helpers;
mod structs;
mod types;

pub use structs::MergeBox;
pub use types::MergeBoxEvent;

#[cfg(test)]
use crate::merge::{Action, MergeFacts, PullState};

#[cfg(test)]
mod tests;
