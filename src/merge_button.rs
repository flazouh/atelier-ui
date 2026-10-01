//! The merge button: a [`ButtonGroup`] of two parts, whose main part says what a press does ("Squash and merge",
//! "Merge when ready", "Add to merge queue", or the first blocker's own action, such as "Ready for
//! review" or "Update branch"), and whose arrow opens a menu of the repository's methods (a check on
//! the chosen one, the default first), merge when ready where the repository allows it, and "Delete
//! branch after merging". What it offers comes from [`crate::merge`]; the owner keeps the reader's
//! [`Choice`] and hears every change of it, so it can remember the method per repository.
//!
//! Blocked with nothing to press, the main part is disabled and says why in its tooltip; the arrow
//! stays live, so the method can still change. Both parts are Tab stops. Enter, Space or Down on the
//! arrow opens the menu on its first row; Up and Down walk the rows, Enter or Space picks one, and
//! Escape closes it, each time with focus back on the arrow. Like every
//! button it is the page inverted, with no colour of its own: a small mark in the success tone beside
//! it says the merge is ready.

mod helpers;
mod structs;
mod types;

pub use structs::MergeButton;
pub use types::{ActionHandler, ChoiceHandler};

#[cfg(test)]
use crate::merge::{Choice, MergeFacts};

#[cfg(test)]
mod tests;
