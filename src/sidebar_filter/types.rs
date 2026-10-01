use crate::{
    session_status::SessionStatus,
    sidebar_model::{SessionData},
};

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
