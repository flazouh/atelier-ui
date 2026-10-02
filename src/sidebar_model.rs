//! The sidebar as data and rules, with no window: which rows show, in which order, what folds, and where
//! the keys move. The view draws what this decides.

mod helpers;
mod structs;
mod types;

pub use helpers::{
    activate, held_order, key_of, position_of, priority_rows, rows, rows_held, since, sorted, step,
};
pub use structs::{Badge, Folds, ProjectData, SessionData, Step};
pub use types::{Activation, Connection, ListMode, Location, Nav, Row, RowKey, Section};

#[cfg(test)]
mod tests;
