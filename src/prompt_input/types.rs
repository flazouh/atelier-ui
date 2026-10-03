use gpui_kit::SharedString;

/// What the user asked for. The panel decides what to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PromptInputEvent {
    /// Send this text. The box is already empty.
    Submit(SharedString),
    /// Stop the running turn.
    Stop,
    /// Hold this text until the running turn ends: ⌘↵ (⌃↵ elsewhere) while it runs. The box is already empty.
    Queue(SharedString),
    /// The ✕ on a queued message: take it out of the queue, by its place in [`PromptInput::set_queued`](crate::prompt_input::PromptInput::set_queued).
    Unqueue(usize),
    /// A queued message's arrow: send it now, into the running turn, by its place.
    SendQueued(usize),
    /// The user chose this action's `value` from the "add to prompt" menu.
    Action(SharedString),
    /// The user picked a different model, by its `value`.
    ModelChanged(SharedString),
    /// The user picked a different mode from [`PromptInput::modes`](crate::prompt_input::PromptInput::modes), by its words.
    ModeChanged(SharedString),
    /// The user pressed the microphone, and the box already shows it listening. The owner starts the capture at once.
    DictationStart,
    /// The user pressed the stop square, or let go. The owner ends the capture and inserts the words.
    DictationStop,
    /// The press was taken back ([`PromptInput::cancel_mic`](crate::prompt_input::PromptInput::cancel_mic)). The owner throws
    /// the recording away.
    DictationCancel,
    /// The ✕ beside the setup was pressed: the owner throws away the words still waiting for the model. The box is idle again.
    DictationDiscard,
    /// The user chose a microphone from the menu beside the microphone button: its `id`, or `None` for the system's default.
    DictationDevice(Option<SharedString>),
    /// The user turned "Hold to record" on or off. On, the microphone records while it is held down and stops when let go.
    DictationHold(bool),
    /// The microphone menu opened. The owner can bring the list up to date with
    /// [`PromptInput::set_voice_devices`](crate::prompt_input::PromptInput::set_voice_devices) before it draws.
    DictationDevices,
    /// A `/` command: its name, from the list after `/` or typed out, and the words after it. The box is empty.
    Command { name: SharedString, args: SharedString },
}

/// The rows of the list: the Task pickers' (28 tall, 2px between), as a Select's.
pub(super) const PICK_ROW: f32 = 28.;

pub(super) const PICK_GAP: f32 = 2.;

pub(super) const PICK_PAD: f32 = 6.;

pub(super) const PICK_MOST: f32 = 256.;

/// Words shown in the box while a press still records.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) enum LiveWords {
    #[default]
    Off,
    /// What was written before they began, and the words now shown after it.
    Showing { base: String, shown: String },
    /// The person edited the box meanwhile: the words stay as they are, and stop moving.
    Left,
}

/// When a message the box gives up goes: at once (into the running turn, if one runs), or after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Sending {
    Now,
    AfterTurn,
}

/// The words on the Send button while a turn runs and the box has text.
pub(super) const STEER_HINT: &str = "Enter to steer, ⌘Enter to queue";

/// A queued row's height, as the Task pickers' rows, and the space between rows.
pub(super) const QUEUED_ROW: f32 = 28.;

pub(super) const QUEUED_GAP: f32 = 2.;
