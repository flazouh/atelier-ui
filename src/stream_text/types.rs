use std::{ops::Range, time::Duration};

/// How long a piece takes to reach full ink: Cursor's `fade-in-fast`, `fade-in .1s ease-in-out`.
pub const FADE: Duration = Duration::from_millis(100);

/// A piece of the tail and its ink, as the view draws it: the bytes, and 0 to 1.
pub type Piece = (Range<usize>, f32);
