//! The task list: tasks grouped by status in Linear's order, a virtual list of 28 px rows, quick filters as
//! chips, and the keyboard first. `j` and `k` or the arrows move, `x` selects, Enter opens, and `s`, `p`,
//! `a`, `i` and `l` change a field of the selected tasks (or of the task under the cursor) through a small
//! picker. The rules are in [`crate::task_list_model`], [`crate::task_keys`] and [`crate::task_edit`].

mod structs;
mod types;

pub use structs::TaskList;
pub use types::TaskListEvent;

#[cfg(test)]
mod tests;
