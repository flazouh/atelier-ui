use gpui_kit::SharedString;

use crate::session_status::SessionStatus;
use super::structs::SessionData;

/// Where a project lives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Location {
    Local,
    Ssh { host: SharedString },
}

/// How a remote project stands. A local project is always `Connected`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connection {
    Connected,
    Connecting,
    Reconnecting,
    Offline,
}

/// One row of the flat list. Every row is the same height, so the list is virtual.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Project { project: usize },
    Session { project: usize, session: usize },
    /// "Show 7 older" when closed, "Show fewer" when open.
    Older { project: usize, hidden: usize, open: bool },
    /// A heading of the priority list, with how many sessions are under it.
    Section { section: Section, count: usize },
    /// "Show 12 more" at the end of the priority list's earlier sessions, "Show fewer" when open.
    MoreEarlier { hidden: usize, open: bool },
}

impl Row {
    /// The project a row belongs to. A heading of the priority list belongs to none: it says 0.
    pub fn project(self) -> usize {
        match self {
            Self::Project { project }
            | Self::Session { project, .. }
            | Self::Older { project, .. } => project,
            Self::Section { .. } | Self::MoreEarlier { .. } => 0,
        }
    }
}

/// How the sidebar lists the sessions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListMode {
    /// Each project with its sessions under it.
    #[default]
    Projects,
    /// One list across the projects, in sections by what the reader owes: the sessions that need them first.
    Priority,
}

impl ListMode {
    pub fn words(self) -> &'static str {
        match self {
            Self::Projects => "Projects",
            Self::Priority => "Priority",
        }
    }

    /// As the settings keep it.
    pub fn key(self) -> &'static str {
        match self {
            Self::Projects => "projects",
            Self::Priority => "priority",
        }
    }

    pub fn from_key(key: Option<&str>) -> Self {
        if key == Some("priority") { Self::Priority } else { Self::Projects }
    }
}

/// The sections of the priority list, top to bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Section {
    /// A yes or no, a question, or a stop: the reader owes the next move.
    NeedsYou,
    /// A turn ended and the reader has not looked yet.
    Finished,
    /// The agent works.
    Working,
    /// Seen, nothing going on.
    Earlier,
}

impl Section {
    pub const ALL: [Section; 4] = [Self::NeedsYou, Self::Finished, Self::Working, Self::Earlier];

    pub fn words(self) -> &'static str {
        match self {
            Self::NeedsYou => "Needs you",
            Self::Finished => "Finished",
            Self::Working => "Working",
            Self::Earlier => "Earlier",
        }
    }

    /// The section a session is in, by its status.
    pub fn of(session: &SessionData) -> Self {
        match session.status {
            SessionStatus::NeedsYou(_) | SessionStatus::Failed(_) => Self::NeedsYou,
            SessionStatus::Finished => Self::Finished,
            SessionStatus::Working => Self::Working,
            SessionStatus::Idle => Self::Earlier,
        }
    }
}

/// A row named by ids, which stay while the rows around it move.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RowKey {
    Project(SharedString),
    Session(SharedString, SharedString),
    Older(SharedString),
    Section(Section),
    MoreEarlier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nav {
    Up,
    Down,
    /// Fold the project, or go to the project from one of its rows.
    Left,
    /// Unfold the project, or go into it.
    Right,
    First,
    Last,
}

/// What Enter does on a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activation {
    OpenSession { project: usize, session: usize },
    ToggleProject(usize),
    ToggleOlder(usize),
    /// "Show more" or "Show fewer" on the priority list's earlier sessions.
    ToggleEarlier,
    /// A heading does nothing.
    Nothing,
}
