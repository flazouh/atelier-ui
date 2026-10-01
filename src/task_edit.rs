//! Changing tasks: the field pickers (`s` status, `p` priority, `a` assignee, `l` labels) and what a choice
//! does to the tasks it acts on. No window: a picker is a list of candidates, a text filter and a cursor.
use gpui_kit::SharedString;

use crate::task_model::{Activity, Assignee, Label, Priority, TaskData, TaskStatus};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Status,
    Priority,
    Assignee,
    Labels,
}

impl Field {
    /// Its place in a row of four, for state kept per field.
    pub fn slot(self) -> usize {
        match self {
            Self::Status => 0,
            Self::Priority => 1,
            Self::Assignee => 2,
            Self::Labels => 3,
        }
    }
    pub fn words(self) -> &'static str {
        match self {
            Self::Status => "Status",
            Self::Priority => "Priority",
            Self::Assignee => "Assignee",
            Self::Labels => "Labels",
        }
    }
}

/// What a choice changes.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    Status(TaskStatus),
    Priority(Priority),
    /// `None` takes the assignee off.
    Assignee(Option<Assignee>),
    /// Puts the label on every task that lacks it, or takes it off when every task has it.
    ToggleLabel(Label),
}

/// One line of a picker.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub change: Change,
    pub words: SharedString,
    /// The tasks it applies to already have this value (for a label: all of them have it).
    pub chosen: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Down,
    Up,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Picker {
    field: Field,
    candidates: Vec<Candidate>,
    query: String,
    /// An index into [`Picker::shown`].
    cursor: usize,
}

impl Picker {
    /// The status picker; `current` is the status all the tasks share, if they do.
    pub fn status(current: Option<TaskStatus>) -> Self {
        let candidates = TaskStatus::BOARD_ORDER
            .into_iter()
            .map(|s| Candidate { change: Change::Status(s), words: s.words().into(), chosen: current == Some(s) })
            .collect();
        Self::new(Field::Status, candidates)
    }

    pub fn priority(current: Option<Priority>) -> Self {
        let candidates = Priority::ALL
            .into_iter()
            .map(|p| Candidate { change: Change::Priority(p), words: p.words().into(), chosen: current == Some(p) })
            .collect();
        Self::new(Field::Priority, candidates)
    }

    /// `people` are the people and agents a task can go to; "Unassigned" is always offered first.
    pub fn assignee(people: &[Assignee], current: Option<&SharedString>) -> Self {
        let mut candidates = vec![Candidate { change: Change::Assignee(None), words: "Unassigned".into(), chosen: current.is_none() }];
        candidates.extend(people.iter().map(|a| Candidate {
            change: Change::Assignee(Some(a.clone())),
            words: a.name().clone(),
            chosen: current == Some(a.name()),
        }));
        Self::new(Field::Assignee, candidates)
    }

    /// `all` are the labels that exist; `on_all` are those every acted-on task already has.
    pub fn labels(all: &[Label], on_all: &[Label]) -> Self {
        let candidates = all
            .iter()
            .map(|l| Candidate { change: Change::ToggleLabel(l.clone()), words: l.name.clone(), chosen: on_all.contains(l) })
            .collect();
        Self::new(Field::Labels, candidates)
    }

    fn new(field: Field, candidates: Vec<Candidate>) -> Self {
        let cursor = 0;
        let mut picker = Self { field, candidates, query: String::new(), cursor };
        // The cursor starts on the value in force, so Enter changes nothing by accident.
        picker.cursor = picker.shown().iter().position(|&i| picker.candidates[i].chosen).unwrap_or(0);
        picker
    }

    pub fn field(&self) -> Field {
        self.field
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    /// The candidates that match the text typed, as indices into [`Picker::candidates`], in order.
    pub fn shown(&self) -> Vec<usize> {
        let needle = self.query.to_lowercase();
        (0..self.candidates.len()).filter(|&i| self.candidates[i].words.to_lowercase().contains(&needle)).collect()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Puts the cursor on the `at`th shown row, as a press on it does.
    pub fn set_cursor(&mut self, at: usize) {
        let count = self.shown().len();
        if count > 0 {
            self.cursor = at.min(count - 1);
        }
    }
    pub fn type_text(&mut self, text: &str) {
        self.query.push_str(text);
        self.cursor = 0;
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.cursor = 0;
    }

    pub fn step(&mut self, step: Step) {
        let count = self.shown().len();
        if count == 0 {
            return;
        }
        self.cursor = match step {
            Step::Down => (self.cursor + 1).min(count - 1),
            Step::Up => self.cursor.saturating_sub(1),
        };
    }

    /// The change the cursor stands on, or `None` when nothing matches.
    pub fn choose(&self) -> Option<Change> {
        self.shown().get(self.cursor).map(|&i| self.candidates[i].change.clone())
    }

    /// A label picker stays open after a choice, so several labels go on in one visit.
    pub fn stays_open(&self) -> bool {
        self.field == Field::Labels
    }

    /// Marks a label candidate as chosen, or not, after it was applied.
    pub fn mark(&mut self, label: &Label, chosen: bool) {
        for c in &mut self.candidates {
            if c.change == Change::ToggleLabel(label.clone()) {
                c.chosen = chosen;
            }
        }
    }
}

/// Applies `change` to the tasks named by `ids`, as `by` at time `now`. Returns how many tasks changed. A
/// task that already has the value is left as it is, and its update time with it.
pub fn apply(tasks: &mut [TaskData], ids: &[SharedString], change: &Change, by: &str, now: u64) -> usize {
    let mut changed = 0;
    let all_have = |tasks: &[TaskData], label: &Label| tasks.iter().filter(|t| ids.contains(&t.id)).all(|t| t.labels.contains(label));
    let removing = matches!(change, Change::ToggleLabel(l) if all_have(tasks, l));
    for task in tasks.iter_mut().filter(|t| ids.contains(&t.id)) {
        let did = match change {
            Change::Status(to) if task.status != *to => {
                task.activity.push(Activity::StatusChanged { by: by.into(), from: task.status, to: *to, at: now });
                task.status = *to;
                true
            }
            Change::Priority(p) if task.priority != *p => {
                task.priority = *p;
                true
            }
            Change::Assignee(who) if task.assignee.as_ref().map(Assignee::name) != who.as_ref().map(Assignee::name) => {
                task.assignee = who.clone();
                true
            }
            Change::ToggleLabel(label) if removing => {
                task.labels.retain(|l| l != label);
                true
            }
            Change::ToggleLabel(label) if !task.labels.contains(label) => {
                task.labels.push(label.clone());
                true
            }
            _ => false,
        };
        if did {
            task.updated_at = now;
            changed += 1;
        }
    }
    changed
}

/// The value all of `tasks` share for a field, for a picker to start on.
pub fn shared_status(tasks: &[&TaskData]) -> Option<TaskStatus> {
    let first = tasks.first()?.status;
    tasks.iter().all(|t| t.status == first).then_some(first)
}

pub fn shared_priority(tasks: &[&TaskData]) -> Option<Priority> {
    let first = tasks.first()?.priority;
    tasks.iter().all(|t| t.priority == first).then_some(first)
}

/// The labels every one of `tasks` has.
pub fn shared_labels(tasks: &[&TaskData]) -> Vec<Label> {
    let Some(first) = tasks.first() else { return Vec::new() };
    first.labels.iter().filter(|l| tasks.iter().all(|t| t.labels.contains(l))).cloned().collect()
}

/// For a shift of status with `]` or `[`: each task's own change, since each moves from where it is. A task at
/// the end of the flow stays.
pub fn shift_status(tasks: &[TaskData], ids: &[SharedString], forward: bool) -> Vec<(SharedString, Change)> {
    ids.iter()
        .filter_map(|id| {
            let task = tasks.iter().find(|t| t.id == *id)?;
            Some((id.clone(), Change::Status(task.status.neighbor(forward)?)))
        })
        .collect()
}

#[cfg(test)]
mod tests;
