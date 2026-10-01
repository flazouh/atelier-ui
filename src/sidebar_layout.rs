//! The sidebar's layout, configured in one place. Everything that decides how the sidebar lists and draws its rows is a
//! field of [`SidebarLayout`]: how it lists (by project or by priority), which sessions (the filter), what a row
//! shows (the agent's icon, the project's badge, the time), and how much it shows before it folds. The sidebar reads
//! its rows and its head from this one value; the Settings page edits it; the settings file keeps it. A new knob is
//! a new field here, with its default, and nowhere else.
use crate::{sidebar_filter::SessionFilter, sidebar_model::ListMode};

/// When a row wears its project's badge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeShow {
    /// Where no project heading says which project it is: in the priority list.
    #[default]
    Auto,
    Always,
    Never,
}

impl BadgeShow {
    pub const ALL: [BadgeShow; 3] = [Self::Auto, Self::Always, Self::Never];

    pub fn words(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Always => "Always",
            Self::Never => "Never",
        }
    }

    /// As the settings keep it.
    pub fn key(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Always => "always",
            Self::Never => "never",
        }
    }

    pub fn from_key(key: Option<&str>) -> Self {
        Self::ALL.into_iter().find(|b| Some(b.key()) == key).unwrap_or_default()
    }
}

/// A project shows this many sessions before the rest fold into "Show N older", unless one that needs the reader
/// would fall beyond them.
pub const FOLD_AFTER: usize = 5;
/// The priority list shows this many earlier sessions before "Show more".
pub const EARLIER_SHOWN: usize = 8;
/// The counts the Settings page offers for those two.
pub const FOLD_CHOICES: [usize; 4] = [3, 5, 8, 12];
pub const EARLIER_CHOICES: [usize; 4] = [5, 8, 12, 20];

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

#[cfg(test)]
mod tests;
