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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoiceInputEvent {
    /// The user pressed the microphone while idle.
    Start,
    /// The user pressed the stop square while listening.
    Stop,
}
