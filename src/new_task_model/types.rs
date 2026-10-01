/// What the dialog's button says it will do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Submit {
    Create,
    /// The assignee is an agent: create the task and start a session for it.
    CreateAndStart,
}
