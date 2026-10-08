use gpui_kit::SharedString;

use crate::task_edit::Change;

/// Columns are built this far past the viewport on each side.
pub(super) const MARGIN: f32 = 200.;

/// How far a card starts above its place after a drop.
pub(super) const DROP_LIFT: f32 = 28.;

/// What the board asks of the app.
#[derive(Clone, Debug, PartialEq)]
pub enum TaskBoardEvent {
    Open(SharedString),
    /// `c`: the reader wants a new task.
    NewTask,
    /// A card was dropped on another column. The board has applied the change to its own copy.
    Changed {
        ids: Vec<SharedString>,
        change: Change,
    },
}
