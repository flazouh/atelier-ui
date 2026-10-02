/// The rows a clipped body shows before it is pressed.
pub const PREVIEW_ROWS: usize = 8;

/// The rows an opened body shows before it scrolls: the review is where the rest is read.
pub(crate) const EXPANDED_ROWS: usize = 24;

/// One row of a diff or a log, in px: a diff's own row height, which the log lines match.
pub(crate) const ROW_HEIGHT: f32 = crate::file_diff::ROW_HEIGHT;
