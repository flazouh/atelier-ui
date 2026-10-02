//! The board: the same tasks as columns by status. A card drags to another column and takes its status; the
//! card settles into place with the Layout spring. Only the columns in view are built (the geometry is the
//! agent panels'), and each column is a virtual list of cards.

mod structs;
mod types;

pub use structs::TaskBoard;
pub use types::TaskBoardEvent;

#[cfg(test)]
mod tests;
