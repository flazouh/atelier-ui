//! CellBar: the one loading and progress bar. A row of small rounded cells that light from the left in step with a number, or,
//! when there is no number, one cell that steps along the row.
//!
//! Anything in atelier that waits on something with a size to it (a download, an upload, a model that loads) draws this, so
//! the waiting looks the same everywhere. It draws only: the owner eases the number, because the owner knows how fast it
//! arrives, and says how long it has been loading when there is no number.
//!
//! - Cells: 4px wide, 6px tall, 2px apart, 1.5px corners by default; [`CellBar::stretch`] shares a row's width among
//!   them instead, for a bar that spans a card.
//! - Lit cells take the bar's color at full strength; the head cell is lit in proportion to the number, so the fill moves
//!   smoothly between two whole cells.
//! - Dark cells are a wash of the text color (`foreground` at 12%), not of the bar's own color, so they show on a light page
//!   and on a dark one alike.
//! - Loading: one cell lit, stepping to the next five times a second. Reduce Motion: it rests in the middle.

mod helpers;
mod structs;
mod types;

pub use helpers::{cell_color, lit, loading_cell, pour, whole_cells};
pub use structs::CellBar;
pub use types::{CELL_GAP, CELL_HEIGHT, CELL_WIDTH, CELLS, DARK_CELL, STEPS_PER_SECOND};

#[cfg(test)]
mod tests;
