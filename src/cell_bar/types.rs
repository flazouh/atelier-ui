/// How many cells, and each one's width, height and the space between, in pixels, by default.
pub const CELLS: usize = 14;

pub const CELL_WIDTH: f32 = 4.;

pub const CELL_HEIGHT: f32 = 6.;

pub const CELL_GAP: f32 = 2.;

/// How many cells the loading light passes in a second.
pub const STEPS_PER_SECOND: f32 = 5.;

/// How strong a dark cell is: the text color at this opacity.
pub const DARK_CELL: f32 = 0.12;

/// How fast a number should ease toward its target, per second, for an owner that smooths it with [`pour`](crate::cell_bar::pour).
pub(super) const POUR_RATE: f32 = 9.;
