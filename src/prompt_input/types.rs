use std::{path::PathBuf, sync::Arc};

use gpui_kit::SharedString;

use crate::icon::IconName;

/// The picture at the front of a chip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChipLook {
    /// No picture: the label alone.
    None,
    /// One of the design system's icons.
    Icon(IconName),
    /// The icon a file of this path gets.
    File(SharedString),
    /// A small picture of the thing itself.
    Image(Arc<gpui_kit::Image>),
}

/// Something on the next message, shown over the text as a small pill with a button that takes it off.
///
/// A chip knows how it looks and nothing about what it means: the owner keeps whatever it stands for, by `id`,
/// and reads it back when the message is sent ([`Message::chips`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chip {
    /// Tells this chip from the others in the box, and ties it to what the owner keeps for it.
    pub id: SharedString,
    pub label: SharedString,
    pub look: ChipLook,
    /// Shown on hover, in place of the label alone.
    pub detail: Option<SharedString>,
    /// Words the message begins with for this chip, such as `@src/lib.rs`. A chip with none adds no words.
    pub mention: Option<SharedString>,
}

impl Chip {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self { id: id.into(), label: label.into(), look: ChipLook::None, detail: None, mention: None }
    }

    /// A file: its name for the label, its path on hover, and `@path` in the message.
    pub fn file(path: impl Into<SharedString>) -> Self {
        let path = path.into();
        let name = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
        Self { id: path.clone(), label: name.into(), look: ChipLook::File(path.clone()), detail: Some(path.clone()), mention: Some(format!("@{path}").into()) }
    }

    pub fn look(mut self, look: ChipLook) -> Self {
        self.look = look;
        self
    }

    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn mention(mut self, mention: impl Into<SharedString>) -> Self {
        self.mention = Some(mention.into());
        self
    }
}

/// What came into the box from outside, by paste or by drop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pasted {
    Text(SharedString),
    Image(Arc<gpui_kit::Image>),
    /// Files, as the clipboard or the drop names them.
    Files(Vec<PathBuf>),
}

/// What the box gives up when it is sent or queued.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Message {
    /// What was written, with each chip's mention in front.
    pub text: SharedString,
    /// The chips that were on it, in the order they were added.
    pub chips: Vec<Chip>,
}

impl Message {
    /// A message of words alone.
    pub fn text(text: impl Into<SharedString>) -> Self {
        Self { text: text.into(), chips: Vec::new() }
    }
}

/// What the user asked for. The panel decides what to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PromptInputEvent {
    /// Send this message. The box is already empty.
    Submit(Message),
    /// Stop the running turn.
    Stop,
    /// Hold this text until the running turn ends: ⌘↵ (⌃↵ elsewhere) while it runs. The box is already empty.
    Queue(Message),
    /// The ✕ on a queued message: take it out of the queue, by its place in [`PromptInput::set_queued`](crate::prompt_input::PromptInput::set_queued).
    Unqueue(usize),
    /// A queued message's arrow: send it now, into the running turn, by its place.
    SendQueued(usize),
    /// The user chose this action's `value` from the "add to prompt" menu.
    Action(SharedString),
    /// The user picked a different model, by its `value`.
    ModelChanged(SharedString),
    /// The user pressed the star of a model in the picker: new sessions start on it, by its `value`.
    ModelStarred(SharedString),
    /// The user pressed the eye of a model in the picker: it leaves the list, by its `value`.
    ModelHidden(SharedString),
    /// The user dragged a model in the picker: the `value`s of every model listed, in the new order.
    ModelsMoved(Vec<SharedString>),
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
    /// Something came in by paste or drop and the box kept it out of the text
    /// ([`PromptInput::set_paste_chips`](crate::prompt_input::PromptInput::set_paste_chips)). The owner decides what it becomes:
    /// usually a chip ([`PromptInput::add_chip`](crate::prompt_input::PromptInput::add_chip)), or text it writes back.
    Paste(Pasted),
    /// A chip was pressed (not its ✕), by its `id`. A quote chip, for one, opens its box again.
    ChipPressed(SharedString),
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
