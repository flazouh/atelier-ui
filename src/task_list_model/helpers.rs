use crate::task_model::{TaskData, TaskStatus};
use super::structs::{Filters, Folds, Group, Sort};
use super::types::Row;

/// The tasks that pass `filters`, grouped by status in `order` and sorted inside each group. A status with
/// no task is left out of a list (`keep_empty` false) and kept as an empty column on a board (true).
pub fn group(tasks: &[TaskData], filters: &Filters, me: &str, sort: Sort, order: &[TaskStatus], keep_empty: bool) -> Vec<Group> {
    let mut groups: Vec<Group> = order.iter().map(|&status| Group { status, tasks: Vec::new() }).collect();
    for (i, task) in tasks.iter().enumerate() {
        if filters.keeps(task, me)
            && let Some(group) = groups.iter_mut().find(|g| g.status == task.status)
        {
            group.tasks.push(i);
        }
    }
    for group in &mut groups {
        group.tasks.sort_by(|&a, &b| sort.compare(&tasks[a], &tasks[b]));
    }
    if !keep_empty {
        groups.retain(|g| !g.tasks.is_empty());
    }
    groups
}

/// The next value of a chip that cycles: none, then each option in turn, then none again. A current
/// value that is not among the options starts over at the first.
pub fn cycle<T: PartialEq + Clone>(current: Option<T>, options: &[T]) -> Option<T> {
    match current {
        None => options.first().cloned(),
        Some(value) => match options.iter().position(|o| *o == value) {
            Some(at) => options.get(at + 1).cloned(),
            None => options.first().cloned(),
        },
    }
}

pub fn rows(groups: &[Group], folds: &Folds) -> Vec<Row> {
    let mut out = Vec::with_capacity(groups.iter().map(|g| g.tasks.len() + 1).sum());
    for group in groups {
        let open = !folds.is_folded(group.status);
        out.push(Row::Header { status: group.status, count: group.tasks.len(), open });
        if open {
            out.extend(group.tasks.iter().map(|&index| Row::Task { index }));
        }
    }
    out
}
