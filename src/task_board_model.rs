//! The board as data: columns by status, where a drag lands, and what a drop changes. The columns' sideways
//! geometry is [`crate::panel_layout::Geometry`], the one the agent panels use, so a board of many columns
//! builds only those in view.

mod helpers;
mod types;

pub use helpers::{
    column_at, columns, drop_on, geometry, move_cursor, spot_of, task_at, visible_cards,
};
pub use types::{BoardMove, CARD_GAP, CARD_HEIGHT, COLUMN_GAP, COLUMN_WIDTH, Spot};

#[cfg(test)]
mod tests;
