//! The geometry of agent panels side by side, with no window: where each column sits, which ones a
//! scroll position shows, where a scroll settles, and how panels group by project. The strip draws what
//! this decides.

mod helpers;
mod structs;
mod types;

pub use helpers::{arrange, columns, edge_fades, fitted, flat, resized};
pub use structs::{Column, Geometry, Group};
pub use types::{DEFAULT_WIDTH, FADE, GAP, GROUP_GAP, MAX_WIDTH, MIN_WIDTH};

#[cfg(test)]
mod tests;
