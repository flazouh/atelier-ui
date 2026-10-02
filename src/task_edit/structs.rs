use gpui_kit::SharedString;

use crate::task_model::{Assignee, Label, Priority, TaskStatus};
use super::types::{Change, Field, Step};

/// One line of a picker.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub change: Change,
    pub words: SharedString,
    /// The tasks it applies to already have this value (for a label: all of them have it).
    pub chosen: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Picker {
    pub(super) field: Field,
    pub(super) candidates: Vec<Candidate>,
    pub(super) query: String,
    /// An index into [`Picker::shown`].
    pub(super) cursor: usize,
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

    pub(super) fn new(field: Field, candidates: Vec<Candidate>) -> Self {
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
