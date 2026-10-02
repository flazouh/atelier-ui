use std::collections::{BTreeSet, HashSet};

use gpui_kit::SharedString;

use crate::{
    task_edit::Change,
    task_model::{Priority, TaskData, TaskStatus},
};
use super::types::{Move, Row, SortKey};

/// The quick filters over a list. Every one that is set must match.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filters {
    /// Assigned to the reader.
    pub mine: bool,
    /// Assigned to this person or agent, by name.
    pub assignee: Option<SharedString>,
    pub label: Option<SharedString>,
    pub priority: Option<Priority>,
    /// Words the title or the key must hold, whatever the case.
    pub text: String,
}

impl Filters {
    /// The filters after the reader chose `change` in a filter picker. Choosing the value already in
    /// force takes it off, and so does "Unassigned", which no task filter can mean. A status is no filter.
    pub fn with(&self, change: &Change) -> Filters {
        let mut next = self.clone();
        match change {
            Change::Assignee(Some(who)) => {
                next.assignee = if self.assignee.as_ref() == Some(who.name()) { None } else { Some(who.name().clone()) };
            }
            Change::Assignee(None) => next.assignee = None,
            Change::Priority(p) => next.priority = if self.priority == Some(*p) { None } else { Some(*p) },
            Change::ToggleLabel(label) => {
                next.label = if self.label.as_ref() == Some(&label.name) { None } else { Some(label.name.clone()) };
            }
            Change::Status(_) => {}
        }
        next
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Whether `task` passes. `me` is the reader's name, for [`Filters::mine`].
    pub fn keeps(&self, task: &TaskData, me: &str) -> bool {
        let assigned = task.assignee.as_ref().map(|a| a.name().as_ref());
        (!self.mine || assigned == Some(me))
            && self.assignee.as_ref().is_none_or(|name| assigned == Some(name.as_ref()))
            && self.label.as_ref().is_none_or(|label| task.labels.iter().any(|l| l.name == *label))
            && self.priority.is_none_or(|p| task.priority == p)
            && (self.text.is_empty() || {
                let needle = self.text.to_lowercase();
                task.title.to_lowercase().contains(&needle) || task.key.to_lowercase().contains(&needle)
            })
    }
}

/// How tasks sort inside a group. Priority puts the most urgent first, the dates the newest first, the
/// title A to Z; `reversed` turns any of them round. Ties fall to the key, so the order is stable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sort {
    pub key: SortKey,
    pub reversed: bool,
}

impl Default for Sort {
    fn default() -> Self {
        Self { key: SortKey::Priority, reversed: false }
    }
}

impl Sort {
    pub fn compare(self, a: &TaskData, b: &TaskData) -> std::cmp::Ordering {
        let by = match self.key {
            // Urgent is the smallest priority, and "no priority" goes last.
            SortKey::Priority => a.priority.cmp(&b.priority).then_with(|| b.updated_at.cmp(&a.updated_at)),
            SortKey::Updated => b.updated_at.cmp(&a.updated_at),
            SortKey::Created => b.created_at.cmp(&a.created_at),
            SortKey::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
        };
        let by = if self.reversed { by.reverse() } else { by };
        by.then_with(|| a.key.cmp(&b.key))
    }
}

/// The tasks of one status, as indices into the task list, sorted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub status: TaskStatus,
    pub tasks: Vec<usize>,
}

/// The groups the reader has folded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Folds(pub(super) HashSet<TaskStatus>);

impl Folds {
    pub fn is_folded(&self, status: TaskStatus) -> bool {
        self.0.contains(&status)
    }

    pub fn set(&mut self, status: TaskStatus, folded: bool) {
        if folded {
            self.0.insert(status);
        } else {
            self.0.remove(&status);
        }
    }

    pub fn toggle(&mut self, status: TaskStatus) {
        let folded = self.is_folded(status);
        self.set(status, !folded);
    }
}

/// Where the cursor is and which tasks are selected. The cursor is a row; the selection is task ids, so it
/// survives a re-sort.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
    pub row: Option<usize>,
    pub selected: BTreeSet<SharedString>,
}

impl Cursor {
    /// Moves the cursor, over headers and tasks alike, and stops at the ends. With no cursor, down takes the
    /// first row and up the last.
    pub fn go(&mut self, rows: &[Row], to: Move) {
        if rows.is_empty() {
            self.row = None;
            return;
        }
        let last = rows.len() - 1;
        self.row = Some(match (self.row.filter(|&r| r <= last), to) {
            (_, Move::First) => 0,
            (_, Move::Last) => last,
            (None, Move::Down) => 0,
            (None, Move::Up) => last,
            (Some(r), Move::Down) => (r + 1).min(last),
            (Some(r), Move::Up) => r.saturating_sub(1),
        });
    }

    /// The task under the cursor.
    pub fn task(&self, rows: &[Row]) -> Option<usize> {
        match rows.get(self.row?)? {
            Row::Task { index } => Some(*index),
            Row::Header { .. } => None,
        }
    }

    /// `x`: selects the task under the cursor, or takes it out of the selection. On a header it selects
    /// every task of the group, or takes them all out when they were all in.
    pub fn toggle(&mut self, rows: &[Row], tasks: &[TaskData]) {
        let Some(row) = self.row.and_then(|r| rows.get(r)).copied() else { return };
        match row {
            Row::Task { index } => {
                let id = tasks[index].id.clone();
                if !self.selected.remove(&id) {
                    self.selected.insert(id);
                }
            }
            Row::Header { .. } => {
                let start = self.row.unwrap_or(0) + 1;
                let members: Vec<&SharedString> =
                    rows[start..].iter().take_while(|r| matches!(r, Row::Task { .. })).filter_map(|r| if let Row::Task { index } = r { Some(&tasks[*index].id) } else { None }).collect();
                if members.iter().all(|id| self.selected.contains(*id)) {
                    members.iter().for_each(|id| {
                        self.selected.remove(*id);
                    });
                } else {
                    self.selected.extend(members.into_iter().cloned());
                }
            }
        }
    }

    /// ⌘A: selects every task in the rows shown, folded groups included.
    pub fn select_all(&mut self, groups: &[Group], tasks: &[TaskData]) {
        self.selected.extend(groups.iter().flat_map(|g| g.tasks.iter().map(|&i| tasks[i].id.clone())));
    }

    /// Esc: clears the selection first, and leaves the cursor for the next Esc.
    pub fn clear(&mut self) -> bool {
        if self.selected.is_empty() {
            return false;
        }
        self.selected.clear();
        true
    }

    /// The tasks a field key acts on: the selection when there is one, else the task under the cursor.
    pub fn acting_on(&self, rows: &[Row], tasks: &[TaskData]) -> Vec<SharedString> {
        if !self.selected.is_empty() {
            return self.selected.iter().cloned().collect();
        }
        self.task(rows).map(|i| vec![tasks[i].id.clone()]).unwrap_or_default()
    }

    /// Keeps the cursor on the same task when the rows change: `id` is the task it was on.
    pub fn follow(&mut self, rows: &[Row], tasks: &[TaskData], id: Option<&SharedString>) {
        if let Some(id) = id {
            self.row = rows.iter().position(|r| matches!(r, Row::Task { index } if tasks[*index].id == *id)).or(self.row.map(|r| r.min(rows.len().saturating_sub(1))));
        }
        if rows.is_empty() {
            self.row = None;
        }
    }
}
