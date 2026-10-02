//! A body that shows only a few rows until the reader presses it: the diff of an edit, the log of a command.
//! It does not scroll while it is clipped. A press opens it to a taller view and tells its owner, which can take the
//! reader to the whole thing (the app opens its review there). A second press folds it back.

mod helpers;
mod structs;
mod types;

pub(crate) use helpers::{hint, window};
pub(crate) use structs::Press;
pub use types::PREVIEW_ROWS;
pub(crate) use types::{EXPANDED_ROWS, ROW_HEIGHT};

#[cfg(test)]
mod tests;
