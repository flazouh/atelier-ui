#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Done,
    Cancelled,
}

/// What to do with row `i` of the previous frame, for the todo at the matching position in the new
/// list. Matching by [`Todo::id`] instead of index means an insertion or removal moves only the rows it
/// actually touches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RowPlan {
    /// Same id, same status and progress: carry the row's motion over unchanged.
    Reuse(usize),
    /// Same id, but status or progress moved on: carry the row over and retarget it.
    Retarget(usize),
    /// No previous row has this id: start a fresh one.
    New,
}
