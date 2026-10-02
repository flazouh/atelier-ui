use crate::new_task_model::Draft;

/// What the dialog asks of the owner.
#[derive(Clone, Debug, PartialEq)]
pub enum NewTaskEvent {
    /// Make the task. `start_session` is true when the assignee is an agent.
    Create { draft: Draft, start_session: bool },
    Cancel,
}
