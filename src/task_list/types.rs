use gpui_kit::{Context, SharedString};

use super::structs::TaskList;
use crate::task_edit::Change;

/// What the list asks of the app.
#[derive(Clone, Debug, PartialEq)]
pub enum TaskListEvent {
    /// The reader opened a task (Enter, or a click).
    Open(SharedString),
    /// A change was made to these tasks. The list has applied it to its own copy; the app saves it.
    Changed {
        ids: Vec<SharedString>,
        change: Change,
    },
    /// `c`: the reader wants a new task.
    NewTask,
}

/// What a press on a filter chip does to the list.
pub(super) type ChipAction = Box<dyn Fn(&mut TaskList, &mut Context<TaskList>)>;
