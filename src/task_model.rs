//! Tasks as plain data, in Linear's model: a status, a priority, an assignee who may be a person or an
//! agent, labels, sub-tasks, and the sessions and pull requests that work on them. atelier-ui fetches none of it;
//! the app hands it over, and the tracker that supplies it (local tasks, Linear, GitHub Issues) comes later.

mod helpers;
mod structs;
mod types;

pub use helpers::sub_tasks;
pub use structs::{Label, SessionLink, TaskData};
pub use types::{Activity, Assignee, Priority, TaskStatus};

#[cfg(test)]
mod tests;
