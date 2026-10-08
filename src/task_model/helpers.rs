use super::structs::TaskData;
use super::types::TaskStatus;

/// The sub-tasks of `task` among `tasks`, and how many of them are done.
pub fn sub_tasks<'a>(tasks: &'a [TaskData], task: &TaskData) -> (Vec<&'a TaskData>, usize) {
    let subs: Vec<&TaskData> = tasks
        .iter()
        .filter(|t| t.parent.as_ref() == Some(&task.id))
        .collect();
    let done = subs.iter().filter(|t| t.status == TaskStatus::Done).count();
    (subs, done)
}
