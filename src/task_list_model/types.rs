use crate::task_model::TaskStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortKey {
    Priority,
    Updated,
    Created,
    Title,
}

impl SortKey {
    pub const ALL: [SortKey; 4] = [Self::Priority, Self::Updated, Self::Created, Self::Title];

    pub fn words(self) -> &'static str {
        match self {
            Self::Priority => "Priority",
            Self::Updated => "Updated",
            Self::Created => "Created",
            Self::Title => "Title",
        }
    }
}

/// One row of the flat list. Every row is the same height, so the list is virtual.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    /// A group's header, with how many tasks it holds and whether it is open.
    Header {
        status: TaskStatus,
        count: usize,
        open: bool,
    },
    /// A task, by its index in the task list.
    Task { index: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Down,
    Up,
    First,
    Last,
}
