use std::collections::HashSet;

use gpui_kit::SharedString;

use crate::{agent_look::AgentLook, session_status::SessionStatus};
use super::types::{Connection, Location};

#[derive(Clone, Debug, PartialEq)]
pub struct SessionData {
    pub id: SharedString,
    /// The first thing the user said. The row cuts it to one line.
    pub title: SharedString,
    /// The agent's mark and its working strip, as data.
    pub look: AgentLook,
    pub status: SessionStatus,
    /// When it last did anything, in seconds since the Unix epoch.
    pub active_at: u64,
    /// The reader put it away: it shows only under the Archived and All filters.
    pub archived: bool,
    /// It has a panel open, so its menu can close it.
    pub in_panel: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectData {
    pub id: SharedString,
    pub name: SharedString,
    pub location: Location,
    pub connection: Connection,
    pub sessions: Vec<SessionData>,
    /// Why the project's pull requests cannot open (no forge remote), or `None` when they can.
    pub pulls_unavailable: Option<SharedString>,
    /// The project's badge: its letters, its colour in the palette, and the image file that stands in for the letters.
    pub badge: Badge,
}

/// What a project's badge shows; see [`crate::project_badge`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Badge {
    pub label: SharedString,
    pub color: usize,
    pub icon: Option<std::path::PathBuf>,
}

/// The folds the reader has made.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Folds {
    pub(super) collapsed: HashSet<SharedString>,
    pub(super) older_open: HashSet<SharedString>,
}

impl Folds {
    pub fn is_collapsed(&self, project: &str) -> bool {
        self.collapsed.contains(project)
    }

    pub fn older_open(&self, project: &str) -> bool {
        self.older_open.contains(project)
    }

    pub fn set_collapsed(&mut self, project: &SharedString, collapsed: bool) {
        if collapsed {
            self.collapsed.insert(project.clone());
        } else {
            self.collapsed.remove(project);
        }
    }

    pub fn toggle_older(&mut self, project: &SharedString) {
        if !self.older_open.remove(project) {
            self.older_open.insert(project.clone());
        }
    }
}

/// What a key does to the selection and to the folds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Step {
    pub select: Option<usize>,
    /// `(project, collapse)`: fold the project or unfold it.
    pub fold: Option<(usize, bool)>,
}
