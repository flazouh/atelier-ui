use gpui_kit::SharedString;

use super::types::{Activity, Assignee, Priority, TaskStatus};
use crate::{agent_look::AgentLook, pr::PrChipData, session_status::SessionStatus};

/// A label: a name and one of the theme's eight tones, by number.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Label {
    pub name: SharedString,
    pub tone: u8,
}

impl Label {
    pub fn new(name: impl Into<SharedString>, tone: u8) -> Self {
        Self {
            name: name.into(),
            tone,
        }
    }
}

/// A session working on a task.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionLink {
    pub id: SharedString,
    pub title: SharedString,
    pub status: SessionStatus,
    pub look: AgentLook,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskData {
    pub id: SharedString,
    /// The short id people say: "LAT-42".
    pub key: SharedString,
    pub title: SharedString,
    /// Markdown.
    pub description: SharedString,
    pub status: TaskStatus,
    pub priority: Priority,
    pub assignee: Option<Assignee>,
    pub labels: Vec<Label>,
    pub project: Option<SharedString>,
    /// The id of the task this is a sub-task of.
    pub parent: Option<SharedString>,
    pub sessions: Vec<SessionLink>,
    pub prs: Vec<PrChipData>,
    pub activity: Vec<Activity>,
    /// Seconds since the Unix epoch.
    pub created_at: u64,
    pub updated_at: u64,
}

impl TaskData {
    /// A task with nothing but a title, an id and a status.
    pub fn new(
        id: impl Into<SharedString>,
        key: impl Into<SharedString>,
        title: impl Into<SharedString>,
        status: TaskStatus,
    ) -> Self {
        Self {
            id: id.into(),
            key: key.into(),
            title: title.into(),
            description: SharedString::default(),
            status,
            priority: Priority::None,
            assignee: None,
            labels: Vec::new(),
            project: None,
            parent: None,
            sessions: Vec::new(),
            prs: Vec::new(),
            activity: Vec::new(),
            created_at: 0,
            updated_at: 0,
        }
    }
}
