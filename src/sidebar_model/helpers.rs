use gpui_kit::SharedString;

use super::structs::{Folds, ProjectData, SessionData, Step};
use super::types::{Activation, Nav, Row, RowKey, Section};

/// The order a project lists its sessions in: those that need the reader first, then the most recent
/// activity first. Equal ones keep the order the app gave them. Gives indices into `sessions`.
pub fn sorted(sessions: &[SessionData]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..sessions.len()).collect();
    order.sort_by_key(|&i| {
        (
            !sessions[i].status.needs_you(),
            std::cmp::Reverse(sessions[i].active_at),
        )
    });
    order
}

/// The order of `now` while the pointer holds the list: the rows keep the order `before` had them in.
/// A session that left gives its place to one that arrived (a past session that opened becomes an open
/// one with a new id), in the order [`sorted`] gives the arrivals; the rest of them go last.
pub fn held_order(before: &[SharedString], now: &[SessionData]) -> Vec<usize> {
    let mut slots: Vec<Option<usize>> = before
        .iter()
        .map(|id| now.iter().position(|s| s.id == *id))
        .collect();
    let known: std::collections::HashSet<usize> = slots.iter().flatten().copied().collect();
    let mut arrived = sorted(now).into_iter().filter(|i| !known.contains(i));
    for slot in slots.iter_mut().filter(|s| s.is_none()) {
        *slot = arrived.next();
    }
    let mut order: Vec<usize> = slots.into_iter().flatten().collect();
    order.extend(arrived);
    order
}

/// The priority list's rows: a heading and its sessions for each section that has any, across every project.
///
/// A row moves only when its status changes, never because the agent writes: "Needs you" lists the one that has waited
/// longest first (that wait is what it costs the reader), "Finished" the newest first, "Working" in the order the
/// sessions began (their ids grow), "Earlier" the newest first, the first `earlier_shown` unless `earlier_open`.
pub fn priority_rows(
    projects: &[ProjectData],
    earlier_open: bool,
    earlier_shown: usize,
) -> Vec<Row> {
    let all: Vec<(usize, usize)> = projects
        .iter()
        .enumerate()
        .flat_map(|(p, d)| (0..d.sessions.len()).map(move |s| (p, s)))
        .collect();
    let session = |&(p, s): &(usize, usize)| &projects[p].sessions[s];
    let mut out = Vec::new();
    for section in Section::ALL {
        let mut mine: Vec<(usize, usize)> = all
            .iter()
            .copied()
            .filter(|at| Section::of(session(at)) == section)
            .collect();
        if mine.is_empty() {
            continue;
        }
        match section {
            Section::NeedsYou => {
                mine.sort_by_key(|at| (session(at).active_at, session(at).id.clone()))
            }
            Section::Finished | Section::Earlier => mine.sort_by_key(|at| {
                (
                    std::cmp::Reverse(session(at).active_at),
                    session(at).id.clone(),
                )
            }),
            Section::Working => mine.sort_by_key(|at| session(at).id.clone()),
        }
        out.push(Row::Section {
            section,
            count: mine.len(),
        });
        let shown = if section == Section::Earlier && !earlier_open {
            mine.len().min(earlier_shown)
        } else {
            mine.len()
        };
        out.extend(
            mine[..shown]
                .iter()
                .map(|&(project, session)| Row::Session { project, session }),
        );
        if section == Section::Earlier && mine.len() > earlier_shown {
            out.push(Row::MoreEarlier {
                hidden: mine.len() - earlier_shown,
                open: earlier_open,
            });
        }
    }
    out
}

/// How many sessions a project shows before the fold: the first [`FOLD_AFTER`](crate::sidebar_layout::FOLD_AFTER), and more if a session
/// that needs the reader or has news would fall beyond them. Those are never folded away.
fn shown_before_fold(sessions: &[SessionData], order: &[usize], fold_after: usize) -> usize {
    let last_news = order
        .iter()
        .rposition(|&i| sessions[i].status.wants_attention())
        .map_or(0, |p| p + 1);
    fold_after.max(last_news).min(order.len())
}

/// The rows, top to bottom.
pub fn rows(projects: &[ProjectData], folds: &Folds) -> Vec<Row> {
    rows_held(projects, folds, None, crate::sidebar_layout::FOLD_AFTER)
}

/// The rows, with each project's sessions in the order `held` keeps for it, while the pointer holds the
/// list.
pub fn rows_held(
    projects: &[ProjectData],
    folds: &Folds,
    held: Option<&std::collections::HashMap<SharedString, Vec<SharedString>>>,
    fold_after: usize,
) -> Vec<Row> {
    let mut out = Vec::with_capacity(projects.iter().map(|p| p.sessions.len() + 2).sum());
    for (project, data) in projects.iter().enumerate() {
        out.push(Row::Project { project });
        if folds.is_collapsed(&data.id) {
            continue;
        }
        if data.sessions.is_empty() {
            continue;
        }
        let order = match held.and_then(|h| h.get(&data.id)) {
            Some(before) => held_order(before, &data.sessions),
            None => sorted(&data.sessions),
        };
        let base = shown_before_fold(&data.sessions, &order, fold_after);
        let open = folds.older_open(&data.id);
        let shown = if open { order.len() } else { base };
        out.extend(
            order[..shown]
                .iter()
                .map(|&session| Row::Session { project, session }),
        );
        if base < order.len() {
            out.push(Row::Older {
                project,
                hidden: order.len() - base,
                open,
            });
        }
    }
    out
}

pub fn key_of(projects: &[ProjectData], row: Row) -> RowKey {
    let id = |project: usize| projects[project].id.clone();
    match row {
        Row::Project { project } => RowKey::Project(id(project)),
        Row::Session { project, session } => {
            RowKey::Session(id(project), projects[project].sessions[session].id.clone())
        }
        Row::Older { project, .. } => RowKey::Older(id(project)),
        Row::Section { section, .. } => RowKey::Section(section),
        Row::MoreEarlier { .. } => RowKey::MoreEarlier,
    }
}

/// The index in `rows` of the row that `key` names, if it is still there.
pub fn position_of(projects: &[ProjectData], rows: &[Row], key: &RowKey) -> Option<usize> {
    rows.iter().position(|&row| &key_of(projects, row) == key)
}

/// Where a key takes the selection, from `at` (the selected row, if any) in `rows`.
pub fn step(
    rows: &[Row],
    folds_collapsed: impl Fn(usize) -> bool,
    at: Option<usize>,
    nav: Nav,
) -> Step {
    if rows.is_empty() {
        return Step::default();
    }
    let last = rows.len() - 1;
    let select = |i: usize| Step {
        select: Some(i),
        fold: None,
    };
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
            Row::Project { project } if !folds_collapsed(project) => Step {
                select: Some(at),
                fold: Some((project, true)),
            },
            Row::Project { .. } => select(at),
            other => select(
                rows.iter()
                    .position(|r| {
                        *r == Row::Project {
                            project: other.project(),
                        }
                    })
                    .unwrap_or(at),
            ),
        },
        Nav::Right => match rows[at] {
            Row::Project { project } if folds_collapsed(project) => Step {
                select: Some(at),
                fold: Some((project, false)),
            },
            Row::Project { .. } => select((at + 1).min(last)),
            _ => select(at),
        },
    }
}

pub fn activate(row: Row) -> Activation {
    match row {
        Row::Project { project } => Activation::ToggleProject(project),
        Row::Session { project, session } => Activation::OpenSession { project, session },
        Row::Older { project, .. } => Activation::ToggleOlder(project),
        Row::MoreEarlier { .. } => Activation::ToggleEarlier,
        Row::Section { .. } => Activation::Nothing,
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
