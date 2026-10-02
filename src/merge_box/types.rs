use gpui_kit::SharedString;

use crate::merge::{Action, Choice};

#[derive(Clone, Debug, PartialEq)]
pub enum MergeBoxEvent {
    /// A press, with the choice it was made under and, for a squash, the commit's words.
    Act { action: Action, choice: Choice, title: SharedString, message: SharedString },
    /// The reader changed the method or a toggle.
    Chose(Choice),
}
