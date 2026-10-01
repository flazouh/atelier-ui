use gpui_kit::SharedString;

use crate::task_model::{Assignee, Label, Priority, TaskStatus};
use super::types::Submit;

/// What the reader has filled in. A draft with an empty title cannot be created.
#[derive(Clone, Debug, PartialEq)]
pub struct Draft {
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub assignee: Option<Assignee>,
    pub labels: Vec<Label>,
    pub project: Option<SharedString>,
    pub parent: Option<SharedString>,
}

impl Default for Draft {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            status: TaskStatus::Todo,
            priority: Priority::None,
            assignee: None,
            labels: Vec::new(),
            project: None,
            parent: None,
        }
    }
}

impl Draft {
    /// The title has something in it besides spaces.
    pub fn can_create(&self) -> bool {
        !self.title.trim().is_empty()
    }

    /// The button's words and what pressing it does.
    pub fn submit(&self) -> Submit {
        if self.assignee.as_ref().is_some_and(Assignee::is_agent) { Submit::CreateAndStart } else { Submit::Create }
    }

    pub fn submit_words(&self) -> &'static str {
        match self.submit() {
            Submit::Create => "Create task",
            Submit::CreateAndStart => "Create and start a session",
        }
    }
}
