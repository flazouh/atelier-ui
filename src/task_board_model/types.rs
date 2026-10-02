/// A column's width and the space between columns.
pub const COLUMN_WIDTH: f32 = 288.;

pub const COLUMN_GAP: f32 = 12.;

/// A card's height, so a column is a virtual list.
pub const CARD_HEIGHT: f32 = 84.;

pub const CARD_GAP: f32 = 8.;

/// A card's place on the board: its column, then its row in the column.
pub type Spot = (usize, usize);

/// A move of the keyboard cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardMove {
    Left,
    Right,
    Up,
    Down,
    /// The first card of the column.
    First,
    /// The last card of the column.
    Last,
}
