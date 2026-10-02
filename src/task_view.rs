//! One task in full: its title, its description (Markdown, edited in a [`CommentComposer`](crate::comment_composer::CommentComposer)), the sub-tasks
//! with their progress, the activity, and a properties rail (status, priority, assignee, labels, project)
//! with the sessions and pull requests linked to it. The rail's fields open the same small pickers as the
//! list, and `s`, `p`, `a`, `i` and `l` work here too.

mod structs;
mod types;

pub use structs::TaskView;
pub use types::TaskViewEvent;
