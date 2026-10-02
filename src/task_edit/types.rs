use crate::task_model::{Assignee, Label, Priority, TaskStatus};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Status,
    Priority,
    Assignee,
    Labels,
}

impl Field {
    /// Its place in a row of four, for state kept per field.
    pub fn slot(self) -> usize {
        match self {
            Self::Status => 0,
            Self::Priority => 1,
            Self::Assignee => 2,
            Self::Labels => 3,
        }
    }
    pub fn words(self) -> &'static str {
        match self {
            Self::Status => "Status",
            Self::Priority => "Priority",
            Self::Assignee => "Assignee",
            Self::Labels => "Labels",
        }
    }
}

/// What a choice changes.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    Status(TaskStatus),
    Priority(Priority),
    /// `None` takes the assignee off.
    Assignee(Option<Assignee>),
    /// Puts the label on every task that lacks it, or takes it off when every task has it.
    ToggleLabel(Label),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Down,
    Up,
}
