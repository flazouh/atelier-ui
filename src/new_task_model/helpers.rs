use gpui_kit::SharedString;

use super::structs::Draft;
use crate::task_model::{Activity, TaskData};

/// The key after the highest number among tasks whose key starts with `prefix-`: "LAT-43" after "LAT-42".
pub fn next_key(prefix: &str, tasks: &[TaskData]) -> SharedString {
    let highest = tasks
        .iter()
        .filter_map(|t| {
            t.key
                .strip_prefix(prefix)?
                .strip_prefix('-')?
                .parse::<u64>()
                .ok()
        })
        .max()
        .unwrap_or(0);
    format!("{prefix}-{}", highest + 1).into()
}

/// The task a draft makes, or `None` when it cannot be created. `id` is the tracker's id for it.
pub fn create(
    draft: &Draft,
    id: impl Into<SharedString>,
    key: SharedString,
    by: &str,
    now: u64,
) -> Option<TaskData> {
    if !draft.can_create() {
        return None;
    }
    let mut task = TaskData::new(id, key, draft.title.trim().to_string(), draft.status);
    task.description = draft.description.trim().to_string().into();
    task.priority = draft.priority;
    task.assignee = draft.assignee.clone();
    task.labels = draft.labels.clone();
    task.project = draft.project.clone();
    task.parent = draft.parent.clone();
    task.created_at = now;
    task.updated_at = now;
    task.activity.push(Activity::Created {
        by: by.into(),
        at: now,
    });
    Some(task)
}
