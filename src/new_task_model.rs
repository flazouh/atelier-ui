//! The new task dialog as data: the draft, when it can be created, and what creating it makes.
use gpui_kit::SharedString;

use crate::task_model::{Activity, Assignee, Label, Priority, TaskData, TaskStatus};

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

/// What the dialog's button says it will do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Submit {
    Create,
    /// The assignee is an agent: create the task and start a session for it.
    CreateAndStart,
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

/// The key after the highest number among tasks whose key starts with `prefix-`: "LAT-43" after "LAT-42".
pub fn next_key(prefix: &str, tasks: &[TaskData]) -> SharedString {
    let highest = tasks
        .iter()
        .filter_map(|t| t.key.strip_prefix(prefix)?.strip_prefix('-')?.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    format!("{prefix}-{}", highest + 1).into()
}

/// The task a draft makes, or `None` when it cannot be created. `id` is the tracker's id for it.
pub fn create(draft: &Draft, id: impl Into<SharedString>, key: SharedString, by: &str, now: u64) -> Option<TaskData> {
    if !draft.can_create() {
        return None;
    }
    let mut task = TaskData::new(id, key, draft.title.trim().to_string(), draft.status);
    task.description = draft.description.trim().to_string().into();
    task.priority = draft.priority;
    task.assignee = draft.assignee.clone();
    task.labels = draft.labels.clone();
    task.project = draft.project.clone();
    task.parent = draft.parent.clone();
    task.created_at = now;
    task.updated_at = now;
    task.activity.push(Activity::Created { by: by.into(), at: now });
    Some(task)
}

#[cfg(test)]
mod tests;
