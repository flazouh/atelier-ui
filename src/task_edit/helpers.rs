use gpui_kit::SharedString;

use crate::task_model::{Activity, Assignee, Label, Priority, TaskData, TaskStatus};
use super::types::Change;

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
