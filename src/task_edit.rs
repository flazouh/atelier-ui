//! Changing tasks: the field pickers (`s` status, `p` priority, `a` assignee, `l` labels) and what a choice
//! does to the tasks it acts on. No window: a picker is a list of candidates, a text filter and a cursor.

mod helpers;
mod structs;
mod types;

pub use helpers::{apply, shared_labels, shared_priority, shared_status, shift_status};
pub use structs::{Candidate, Picker};
pub use types::{Change, Field, Step};

#[cfg(test)]
mod tests;
