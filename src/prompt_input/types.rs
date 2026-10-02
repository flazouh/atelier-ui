use gpui_kit::SharedString;

/// What the user asked for. The panel decides what to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PromptInputEvent {
    /// Send this text. The box is already empty.
    Submit(SharedString),
    /// Stop the running turn.
    Stop,
    /// The user chose this action's `value` from the "add to prompt" menu.
    Action(SharedString),
    /// The user picked a different model, by its `value`.
    ModelChanged(SharedString),
    /// The user picked a different mode from [`PromptInput::modes`](crate::prompt_input::PromptInput::modes), by its words.
    ModeChanged(SharedString),
    /// A `/` command: its name, from the list after `/` or typed out, and the words after it. The box is empty.
    Command { name: SharedString, args: SharedString },
}

/// The rows of the list: the Task pickers' (28 tall, 2px between), as a Select's.
pub(super) const PICK_ROW: f32 = 28.;

pub(super) const PICK_GAP: f32 = 2.;

pub(super) const PICK_PAD: f32 = 6.;

pub(super) const PICK_MOST: f32 = 256.;
