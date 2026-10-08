//! The whole dictation bar: a microphone that turns into a stop button, with amber bars beside it while it listens.
//!
//! SPIKE, with [`crate::voice_waves`] and [`crate::voice_setup`]. The owner runs the microphone and the model; this is
//! the face of it, and it says what the user did.
//!
//! - Idle: a hint at the left and a round microphone at the right.
//! - First press: the owner finds the speech model missing and calls [`VoiceInput::set_setup`]. The hint morphs into
//!   the setup bar ([`VoiceSetup`](crate::voice_setup::VoiceSetup)), which the owner feeds with the download, and the microphone waits.
//! - Listening: the setup (or the hint) morphs into rounded amber bars that follow the voice, with the time recorded at
//!   their right. The microphone cross-fades into a stop square on an amber disc, the way the prompt input swaps send
//!   for stop (`Spring::SWAP`, the outgoing glyph rising 3px as it fades, the incoming one falling in), and a soft ring
//!   breathes out from the disc.
//! - Stop: all of it runs back. The bars morph into the hint, the stop square into the microphone.
//! - Reduce Motion: the glyphs swap at once, the ring is left out and the bars hold still.
//!
//! The owner plays the sound cues on [`VoiceInputEvent`]s, before it changes the mode, so the cue lands with the press.

mod helpers;
mod structs;
mod types;

pub use helpers::{clock, ring_at, ring_phase};
pub(crate) use helpers::{failed_row, key, listening_row, mic_slot};
pub(crate) use structs::Mic;
pub use structs::VoiceInput;
pub use types::{
    BAR_HEIGHT, BUTTON, RING_REACH, RING_SECONDS, VoiceDevice, VoiceInputEvent, VoiceMode,
};

#[cfg(test)]
mod tests;
