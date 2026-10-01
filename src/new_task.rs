//! The create dialog: a title, a description, four property buttons (status, priority, assignee, labels),
//! and one button that says what it will do. When the assignee is an agent the button reads "Create and
//! start a session". The owner opens it (the list's `c`), and hears [`NewTaskEvent`].

gpui_kit::actions!(new_task, [SubmitTask]);

mod helpers;
mod structs;
mod types;

pub(crate) use helpers::bind_keys;
pub use structs::NewTask;
pub use types::NewTaskEvent;
