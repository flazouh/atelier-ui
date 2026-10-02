//! One task on one line, in the 28 px Small scale: priority, key, status, title, labels, the session and
//! the pull request working on it, the assignee, and the time since the last change. Dense, and virtual in
//! lists (every row is the same height).

mod helpers;
mod structs;
mod types;

pub use helpers::{assignee_mark, label_chip, shown_labels};
pub use structs::TaskRow;
pub use types::{MAX_LABELS, ROW_HEIGHT};

#[cfg(test)]
mod tests;
