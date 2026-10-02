//! The task list and the board as data and rules, with no window: which tasks a filter keeps, how they
//! sort, how they group by status, which rows the list shows, and where the cursor and the selection
//! go. The views draw what this decides.

mod helpers;
mod structs;
mod types;

pub use helpers::{cycle, group, rows};
pub use structs::{Cursor, Filters, Folds, Group, Sort};
pub use types::{Move, Row, SortKey};

#[cfg(test)]
mod tests;
