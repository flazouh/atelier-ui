/// beui's `maxHeight` for the diff viewport.
pub(super) const MAX_HEIGHT: f32 = 220.;

/// Every row's height: the virtual list counts on it.
pub const ROW_HEIGHT: f32 = 20.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
    /// A `@@ -1,4 +1,5 @@` hunk header. beui's own `FileDiffLine` has no such row (its callers hand it
    /// pre-split lines); this crate parses raw unified diff text instead, so hunk headers need a row.
    Hunk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileDiffStatus {
    Streaming,
    Complete,
}
