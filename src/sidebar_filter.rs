//! Which sessions the sidebar lists: a filter by what the reader asks of the list (the sessions that wait for
//! them, the ones at work, the ones put away). Pure: the app narrows the projects with
//! [`narrow`] before it hands them to the sidebar.
use gpui_kit::SharedString;

use crate::{session_status::SessionStatus, sidebar_model::{ProjectData, SessionData}};

/// What the list shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SessionFilter {
    /// Every session that is not archived: the list as it always was.
    #[default]
    Active,
    /// The sessions that wait for the reader: a yes or no, a question, or a finished turn not yet seen.
    NeedsYou,
    /// The sessions whose agent works now.
    Working,
    /// Only the sessions the reader archived.
    Archived,
    /// Every session, archived or not.
    All,
}

impl SessionFilter {
    pub const ALL: [SessionFilter; 5] = [Self::Active, Self::NeedsYou, Self::Working, Self::Archived, Self::All];

    pub fn words(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::NeedsYou => "Needs you",
            Self::Working => "Working",
            Self::Archived => "Archived",
            Self::All => "All sessions",
        }
    }

    /// The name a test finds the filter's menu row by.
    pub fn row(self) -> &'static str {
        match self {
            Self::Active => "filter-active",
            Self::NeedsYou => "filter-needs-you",
            Self::Working => "filter-working",
            Self::Archived => "filter-archived",
            Self::All => "filter-all",
        }
    }

    /// Whether `session` is on the list under this filter.
    pub fn keeps(self, session: &SessionData) -> bool {
        match self {
            Self::Active => !session.archived,
            Self::NeedsYou => !session.archived && session.status.wants_attention(),
            Self::Working => !session.archived && session.status == SessionStatus::Working,
            Self::Archived => session.archived,
            Self::All => true,
        }
    }

    /// Whether this is the list as it always was, so the filter button needs no mark.
    pub fn is_default(self) -> bool {
        self == Self::Active
    }
}

/// The projects with only the sessions that pass `filter`. A project stays even with none left.
pub fn narrow(projects: &[ProjectData], filter: SessionFilter) -> Vec<ProjectData> {
    projects
        .iter()
        .map(|project| {
            let mut project = project.clone();
            project.sessions.retain(|s| filter.keeps(s));
            project
        })
        .collect()
}

/// How many sessions of `projects` the filter hides, for a line such as "3 archived".
pub fn hidden_by(projects: &[ProjectData], filter: SessionFilter) -> usize {
    projects.iter().flat_map(|p| p.sessions.iter()).filter(|s| !filter.keeps(s)).count()
}

/// The words for the filter button's tooltip.
pub fn describe(filter: SessionFilter) -> SharedString {
    if filter.is_default() { "Filter sessions".into() } else { format!("Showing: {}", filter.words()).into() }
}

#[cfg(test)]
mod tests;
