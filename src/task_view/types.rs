use gpui_kit::SharedString;

use crate::task_edit::Change;

/// The rail's width.
pub(super) const RAIL: f32 = 264.;

/// What the view asks of the app.
#[derive(Clone, Debug, PartialEq)]
pub enum TaskViewEvent {
    /// A field changed. The view has applied it to its own copy; the app saves it.
    Changed { id: SharedString, change: Change },
    DescriptionSaved { id: SharedString, text: SharedString },
    Commented { id: SharedString, text: SharedString },
    OpenSession(SharedString),
    OpenPr(u64),
    OpenTask(SharedString),
    /// The reader asked for a session for a task assigned to an agent.
    StartSession(SharedString),
}
