use crate::{sidebar_filter::SessionFilter, sidebar_model::ListMode};
use super::types::{BadgeShow, EARLIER_SHOWN, FOLD_AFTER};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SidebarLayout {
    /// By project, or one list by priority. Chosen in the head.
    pub mode: ListMode,
    /// Which sessions. Chosen in the head; it starts as Active each launch, so no session is hidden by a forgotten choice.
    pub filter: SessionFilter,
    /// When a row wears its project's badge.
    pub project_badge: BadgeShow,
    /// Whether a row shows the time since the session last did anything. What it owes the reader ("Needs approval") always shows.
    pub show_time: bool,
    /// Whether a row shows the agent's icon.
    pub show_agent_icon: bool,
    /// Sessions a project shows before it folds the rest.
    pub fold_after: usize,
    /// Earlier sessions the priority list shows before "Show more".
    pub earlier_shown: usize,
}

impl Default for SidebarLayout {
    fn default() -> Self {
        Self {
            mode: ListMode::default(),
            filter: SessionFilter::default(),
            project_badge: BadgeShow::default(),
            show_time: true,
            show_agent_icon: true,
            fold_after: FOLD_AFTER,
            earlier_shown: EARLIER_SHOWN,
        }
    }
}

impl SidebarLayout {
    /// This layout with the look (what a row shows, how much folds) of `other`, and its own mode and filter: the head
    /// chooses those two, the Settings page the rest.
    pub fn with_look_of(self, other: &SidebarLayout) -> SidebarLayout {
        SidebarLayout { mode: self.mode, filter: self.filter, ..*other }
    }

    /// Whether the rows wear the project's badge in the mode now in force.
    pub fn badge_on_rows(&self) -> bool {
        match self.project_badge {
            BadgeShow::Auto => self.mode == ListMode::Priority,
            BadgeShow::Always => true,
            BadgeShow::Never => false,
        }
    }

    /// Whether rows sit at the list's edge: the priority list has no project heading to indent under.
    pub fn rows_flush(&self) -> bool {
        self.mode == ListMode::Priority
    }
}
