use gpui_kit::SharedString;

/// The bar's height and the round button's size, in pixels.
pub const BAR_HEIGHT: f32 = 44.;

pub const BUTTON: f32 = 28.;

/// How far the ring grows past the disc, in pixels, and how long one breath takes, in seconds.
pub const RING_REACH: f32 = 9.;

pub const RING_SECONDS: f32 = 1.4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoiceMode {
    Idle,
    /// The speech model is being fetched or loaded; see [`VoiceInput::set_setup`](crate::voice_input::VoiceInput::set_setup).
    Setup,
    Listening,
    /// The last press ended without words; the bar says why until the owner sets another mode. The microphone can be pressed.
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoiceInputEvent {
    /// The user pressed the microphone while idle.
    Start,
    /// The user pressed the stop square while listening.
    Stop,
}

/// One microphone the owner offers in the menu beside the microphone button.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceDevice {
    /// What the owner knows it by; handed back in [`PromptInputEvent::DictationDevice`](crate::prompt_input::PromptInputEvent::DictationDevice).
    pub id: SharedString,
    pub label: SharedString,
}

impl VoiceDevice {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self { id: id.into(), label: label.into() }
    }
}
