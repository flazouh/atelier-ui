//! Tasks as plain data, in Linear's model: a status, a priority, an assignee who may be a person or an
//! agent, labels, sub-tasks, and the sessions and pull requests that work on them. atelier-ui fetches none of it;
//! the app hands it over, and the tracker that supplies it (local tasks, Linear, GitHub Issues) comes later.
use gpui_kit::SharedString;

use crate::{agent_look::AgentLook, pr::PrChipData, session_status::SessionStatus};

/// Where a task stands. Linear's default workflow: Backlog, Todo, In Progress, Done and Canceled, with In
/// Review between In Progress and Done.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TaskStatus {
    Backlog,
    Todo,
    InProgress,
    InReview,
    Done,
    Canceled,
}

impl TaskStatus {
    /// The order of a list's groups, as Linear sorts by status: the work closest to done first.
    pub const LIST_ORDER: [TaskStatus; 6] =
        [Self::InProgress, Self::InReview, Self::Todo, Self::Backlog, Self::Done, Self::Canceled];

    /// The order of a board's columns, from the start of the work to the end.
    pub const BOARD_ORDER: [TaskStatus; 6] =
        [Self::Backlog, Self::Todo, Self::InProgress, Self::InReview, Self::Done, Self::Canceled];

    /// The status after (or before) this one in the flow of the board. `None` at the ends.
    pub fn neighbor(self, forward: bool) -> Option<TaskStatus> {
        let at = Self::BOARD_ORDER.iter().position(|s| *s == self)?;
        if forward { Self::BOARD_ORDER.get(at + 1).copied() } else { at.checked_sub(1).map(|i| Self::BOARD_ORDER[i]) }
    }

    pub fn words(self) -> &'static str {
        match self {
            Self::Backlog => "Backlog",
            Self::Todo => "Todo",
            Self::InProgress => "In Progress",
            Self::InReview => "In Review",
            Self::Done => "Done",
            Self::Canceled => "Canceled",
        }
    }

    /// Work is still to do or going on.
    pub fn is_open(self) -> bool {
        !matches!(self, Self::Done | Self::Canceled)
    }

    /// How much of the work the status says is done, for a mark that fills: 0 to 1.
    pub fn progress(self) -> f32 {
        match self {
            Self::Backlog | Self::Todo | Self::Canceled => 0.,
            Self::InProgress => 0.5,
            Self::InReview => 0.75,
            Self::Done => 1.,
        }
    }
}

/// How soon a task matters. Urgent sorts first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Priority {
    Urgent,
    High,
    Medium,
    Low,
    None,
}

impl Priority {
    pub const ALL: [Priority; 5] = [Self::Urgent, Self::High, Self::Medium, Self::Low, Self::None];

    pub fn words(self) -> &'static str {
        match self {
            Self::Urgent => "Urgent",
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
            Self::None => "No priority",
        }
    }

    /// How many of a priority mark's three bars are filled: 0 for none.
    pub fn bars(self) -> usize {
        match self {
            Self::Urgent | Self::None => 0,
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
        }
    }
}

/// A label: a name and one of the theme's eight tones, by number.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Label {
    pub name: SharedString,
    pub tone: u8,
}

impl Label {
    pub fn new(name: impl Into<SharedString>, tone: u8) -> Self {
        Self { name: name.into(), tone }
    }
}

/// Who a task is assigned to: a person, or an agent.
#[derive(Clone, Debug, PartialEq)]
pub enum Assignee {
    Person { name: SharedString },
    /// The look is boxed: it is large, and most assignees are people.
    Agent { name: SharedString, look: Box<AgentLook> },
}

impl Assignee {
    pub fn agent(name: impl Into<SharedString>, look: AgentLook) -> Self {
        Self::Agent { name: name.into(), look: Box::new(look) }
    }

    /// The name, which is also the id: two assignees with one name are one.
    pub fn name(&self) -> &SharedString {
        match self {
            Self::Person { name } | Self::Agent { name, .. } => name,
        }
    }

    pub fn is_agent(&self) -> bool {
        matches!(self, Self::Agent { .. })
    }

    /// The first letter of a person's name, in capitals, for the round mark.
    pub fn initial(&self) -> String {
        self.name().chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default()
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

/// One entry of a task's activity.
#[derive(Clone, Debug, PartialEq)]
pub enum Activity {
    Comment { author: SharedString, text: SharedString, at: u64 },
    StatusChanged { by: SharedString, from: TaskStatus, to: TaskStatus, at: u64 },
    SessionStarted { agent: SharedString, at: u64 },
    PrOpened { number: u64, at: u64 },
    PrMerged { number: u64, at: u64 },
    Created { by: SharedString, at: u64 },
    /// A commit the work made, by its short id and its first line.
    Committed { by: SharedString, sha: SharedString, subject: SharedString, at: u64 },
}

impl Activity {
    pub fn at(&self) -> u64 {
        match self {
            Self::Comment { at, .. }
            | Self::StatusChanged { at, .. }
            | Self::SessionStarted { at, .. }
            | Self::PrOpened { at, .. }
            | Self::PrMerged { at, .. }
            | Self::Created { at, .. }
            | Self::Committed { at, .. } => *at,
        }
    }

    /// The entry as one line, for the activity list and a screen reader.
    pub fn words(&self) -> String {
        match self {
            Self::Comment { author, .. } => format!("{author} commented"),
            Self::StatusChanged { by, from, to, .. } => format!("{by} changed the status from {} to {}", from.words(), to.words()),
            Self::SessionStarted { agent, .. } => format!("{agent} started a session"),
            Self::PrOpened { number, .. } => format!("PR #{number} opened"),
            Self::PrMerged { number, .. } => format!("PR #{number} merged"),
            Self::Created { by, .. } => format!("{by} created the task"),
            Self::Committed { sha, subject, .. } => format!("{sha} {subject}"),
        }
    }
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
    pub fn new(id: impl Into<SharedString>, key: impl Into<SharedString>, title: impl Into<SharedString>, status: TaskStatus) -> Self {
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

/// The sub-tasks of `task` among `tasks`, and how many of them are done.
pub fn sub_tasks<'a>(tasks: &'a [TaskData], task: &TaskData) -> (Vec<&'a TaskData>, usize) {
    let subs: Vec<&TaskData> = tasks.iter().filter(|t| t.parent.as_ref() == Some(&task.id)).collect();
    let done = subs.iter().filter(|t| t.status == TaskStatus::Done).count();
    (subs, done)
}

#[cfg(test)]
mod tests;
