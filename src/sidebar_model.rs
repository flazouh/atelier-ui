//! The sidebar as data and rules, with no window: which rows show, in which order, what folds, and where
//! the keys move. The view draws what this decides.
use std::collections::HashSet;

use gpui_kit::SharedString;

use crate::{agent_look::AgentLook, session_status::SessionStatus};


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
    pub branch: Option<SharedString>,
    pub sessions: Vec<SessionData>,
    /// Why the project's pull requests cannot open (no forge remote), or `None` when they can.
    pub pulls_unavailable: Option<SharedString>,
    /// How many tasks are open (not Done, not Canceled), once the project has read its tasks.
    pub tasks_open: Option<usize>,
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

/// The order a project lists its sessions in: those that need the reader first, then the most recent
/// activity first. Equal ones keep the order the app gave them. Gives indices into `sessions`.
pub fn sorted(sessions: &[SessionData]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..sessions.len()).collect();
    order.sort_by_key(|&i| (!sessions[i].status.needs_you(), std::cmp::Reverse(sessions[i].active_at)));
    order
}

/// The order of `now` while the pointer holds the list: the rows keep the order `before` had them in.
/// A session that left gives its place to one that arrived (a past session that opened becomes an open
/// one with a new id), in the order [`sorted`] gives the arrivals; the rest of them go last.
pub fn held_order(before: &[SharedString], now: &[SessionData]) -> Vec<usize> {
    let mut slots: Vec<Option<usize>> = before.iter().map(|id| now.iter().position(|s| s.id == *id)).collect();
    let known: std::collections::HashSet<usize> = slots.iter().flatten().copied().collect();
    let mut arrived = sorted(now).into_iter().filter(|i| !known.contains(i));
    for slot in slots.iter_mut().filter(|s| s.is_none()) {
        *slot = arrived.next();
    }
    let mut order: Vec<usize> = slots.into_iter().flatten().collect();
    order.extend(arrived);
    order
}
/// The folds the reader has made.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Folds {
    collapsed: HashSet<SharedString>,
    older_open: HashSet<SharedString>,
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

/// One row of the flat list. Every row is the same height, so the list is virtual.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Project { project: usize },
    Session { project: usize, session: usize },
    /// "Show 7 older" when closed, "Show fewer" when open.
    Older { project: usize, hidden: usize, open: bool },
    /// A project with no session.
    Empty { project: usize },
    /// The project's tasks, at the end of its section.
    Tasks { project: usize },
    /// A heading of the priority list, with how many sessions are under it.
    Section { section: Section, count: usize },
    /// "Show 12 more" at the end of the priority list's earlier sessions, "Show fewer" when open.
    MoreEarlier { hidden: usize, open: bool },
}

impl Row {
    /// The project a row belongs to. A heading of the priority list belongs to none: it says 0.
    pub fn project(self) -> usize {
        match self {
            Self::Project { project } | Self::Session { project, .. } | Self::Older { project, .. } | Self::Tasks { project } | Self::Empty { project } => project,
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

/// The priority list's rows: a heading and its sessions for each section that has any, across every project.
///
/// A row moves only when its status changes, never because the agent writes: "Needs you" lists the one that has waited
/// longest first (that wait is what it costs the reader), "Finished" the newest first, "Working" in the order the
/// sessions began (their ids grow), "Earlier" the newest first, the first `earlier_shown` unless `earlier_open`.
pub fn priority_rows(projects: &[ProjectData], earlier_open: bool, earlier_shown: usize) -> Vec<Row> {
    let all: Vec<(usize, usize)> = projects.iter().enumerate().flat_map(|(p, d)| (0..d.sessions.len()).map(move |s| (p, s))).collect();
    let session = |&(p, s): &(usize, usize)| &projects[p].sessions[s];
    let mut out = Vec::new();
    for section in Section::ALL {
        let mut mine: Vec<(usize, usize)> = all.iter().copied().filter(|at| Section::of(session(at)) == section).collect();
        if mine.is_empty() {
            continue;
        }
        match section {
            Section::NeedsYou => mine.sort_by_key(|at| (session(at).active_at, session(at).id.clone())),
            Section::Finished | Section::Earlier => mine.sort_by_key(|at| (std::cmp::Reverse(session(at).active_at), session(at).id.clone())),
            Section::Working => mine.sort_by_key(|at| session(at).id.clone()),
        }
        out.push(Row::Section { section, count: mine.len() });
        let shown = if section == Section::Earlier && !earlier_open { mine.len().min(earlier_shown) } else { mine.len() };
        out.extend(mine[..shown].iter().map(|&(project, session)| Row::Session { project, session }));
        if section == Section::Earlier && mine.len() > earlier_shown {
            out.push(Row::MoreEarlier { hidden: mine.len() - earlier_shown, open: earlier_open });
        }
    }
    out
}

/// How many sessions a project shows before the fold: the first [`FOLD_AFTER`], and more if a session
/// that needs the reader or has news would fall beyond them. Those are never folded away.
fn shown_before_fold(sessions: &[SessionData], order: &[usize], fold_after: usize) -> usize {
    let last_news = order.iter().rposition(|&i| sessions[i].status.wants_attention()).map_or(0, |p| p + 1);
    fold_after.max(last_news).min(order.len())
}

/// The rows, top to bottom.
pub fn rows(projects: &[ProjectData], folds: &Folds) -> Vec<Row> {
    rows_held(projects, folds, None, crate::sidebar_layout::FOLD_AFTER)
}
/// The rows, with each project's sessions in the order `held` keeps for it, while the pointer holds the
/// list.
pub fn rows_held(projects: &[ProjectData], folds: &Folds, held: Option<&std::collections::HashMap<SharedString, Vec<SharedString>>>, fold_after: usize) -> Vec<Row> {
    let mut out = Vec::with_capacity(projects.iter().map(|p| p.sessions.len() + 2).sum());
    for (project, data) in projects.iter().enumerate() {
        out.push(Row::Project { project });
        if folds.is_collapsed(&data.id) {
            continue;
        }
        if data.sessions.is_empty() {
            out.push(Row::Empty { project });
            out.push(Row::Tasks { project });
            continue;
        }
        let order = match held.and_then(|h| h.get(&data.id)) {
            Some(before) => held_order(before, &data.sessions),
            None => sorted(&data.sessions),
        };
        let base = shown_before_fold(&data.sessions, &order, fold_after);
        let open = folds.older_open(&data.id);
        let shown = if open { order.len() } else { base };
        out.extend(order[..shown].iter().map(|&session| Row::Session { project, session }));
        if base < order.len() {
            out.push(Row::Older { project, hidden: order.len() - base, open });
        }
        out.push(Row::Tasks { project });
    }
    out
}

/// A row named by ids, which stay while the rows around it move.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RowKey {
    Project(SharedString),
    Session(SharedString, SharedString),
    Older(SharedString),
    Empty(SharedString),
    Tasks(SharedString),
    Section(Section),
    MoreEarlier,
}

pub fn key_of(projects: &[ProjectData], row: Row) -> RowKey {
    let id = |project: usize| projects[project].id.clone();
    match row {
        Row::Project { project } => RowKey::Project(id(project)),
        Row::Session { project, session } => RowKey::Session(id(project), projects[project].sessions[session].id.clone()),
        Row::Older { project, .. } => RowKey::Older(id(project)),
        Row::Empty { project } => RowKey::Empty(id(project)),
        Row::Tasks { project } => RowKey::Tasks(id(project)),
        Row::Section { section, .. } => RowKey::Section(section),
        Row::MoreEarlier { .. } => RowKey::MoreEarlier,
    }
}

/// The index in `rows` of the row that `key` names, if it is still there.
pub fn position_of(projects: &[ProjectData], rows: &[Row], key: &RowKey) -> Option<usize> {
    rows.iter().position(|&row| &key_of(projects, row) == key)
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

/// What a key does to the selection and to the folds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Step {
    pub select: Option<usize>,
    /// `(project, collapse)`: fold the project or unfold it.
    pub fold: Option<(usize, bool)>,
}

/// Where a key takes the selection, from `at` (the selected row, if any) in `rows`.
pub fn step(rows: &[Row], folds_collapsed: impl Fn(usize) -> bool, at: Option<usize>, nav: Nav) -> Step {
    if rows.is_empty() {
        return Step::default();
    }
    let last = rows.len() - 1;
    let select = |i: usize| Step { select: Some(i), fold: None };
    let Some(at) = at.filter(|&i| i <= last) else {
        return match nav {
            Nav::Up | Nav::Last => select(last),
            _ => select(0),
        };
    };
    match nav {
        Nav::Up => select(at.saturating_sub(1)),
        Nav::Down => select((at + 1).min(last)),
        Nav::First => select(0),
        Nav::Last => select(last),
        Nav::Left => match rows[at] {
            Row::Project { project } if !folds_collapsed(project) => Step { select: Some(at), fold: Some((project, true)) },
            Row::Project { .. } => select(at),
            other => select(rows.iter().position(|r| *r == Row::Project { project: other.project() }).unwrap_or(at)),
        },
        Nav::Right => match rows[at] {
            Row::Project { project } if folds_collapsed(project) => Step { select: Some(at), fold: Some((project, false)) },
            Row::Project { .. } => select((at + 1).min(last)),
            _ => select(at),
        },
    }
}

/// What Enter does on a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activation {
    OpenSession { project: usize, session: usize },
    ToggleProject(usize),
    ToggleOlder(usize),
    NewSession(usize),
    OpenTasks(usize),
    /// "Show more" or "Show fewer" on the priority list's earlier sessions.
    ToggleEarlier,
    /// A heading does nothing.
    Nothing,
}

pub fn activate(row: Row) -> Activation {
    match row {
        Row::Project { project } => Activation::ToggleProject(project),
        Row::Session { project, session } => Activation::OpenSession { project, session },
        Row::Older { project, .. } => Activation::ToggleOlder(project),
        Row::Empty { project } => Activation::NewSession(project),
        Row::Tasks { project } => Activation::OpenTasks(project),
        Row::MoreEarlier { .. } => Activation::ToggleEarlier,
        Row::Section { .. } => Activation::Nothing,
    }
}

/// The words of a project's tasks row: "Tasks", and the count when there are open tasks.
pub fn tasks_words(open: Option<usize>) -> String {
    match open {
        Some(n) if n > 0 => format!("Tasks {n}"),
        _ => "Tasks".into(),
    }
}

/// The time since `then`, in one unit and no more: "now", "2m", "3h", "5d", "8w".
pub fn since(now: u64, then: u64) -> String {
    let seconds = now.saturating_sub(then);
    match seconds {
        0..=44 => "now".into(),
        45..=3599 => format!("{}m", (seconds + 30) / 60),
        3600..=86_399 => format!("{}h", seconds / 3600),
        86_400..=1_209_599 => format!("{}d", seconds / 86_400),
        _ => format!("{}w", seconds / 604_800),
    }
}

#[cfg(test)]
mod tests;
