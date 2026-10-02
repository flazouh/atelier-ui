//! beui's PromptInput (`components/agents/prompt-input.tsx`): where the user writes to the agent.
//!
//! - Frame: `rounded-2xl bg-card p-2`, dimmed to 60% opacity while `disabled`. The textarea auto-grows
//!   between `min_rows` and `max_rows` (defaults 2 and 8) at `text-sm leading-6` (24px lines).
//! - Action row (`mt-1 min-h-8 gap-1`): a round `size-8` ghost Plus button that opens the "add to
//!   prompt" menu (its icon turns 45° on `Spring::SWAP`), the model picker (beui's `Select`, in its
//!   `compact` chip skin: `h-8 rounded-xl text-xs`, no chevron, borderless, `shadow-none` panel), a
//!   flexible spacer, and the round `size-8` send button.
//! - Send↔Stop: a single `send_swap` channel drives both icons in the same slot: the outgoing one
//!   fades out rising 3px, the incoming one fades in falling from 3px; beui's `SPRING_SWAP` cross-fade
//!   without the scale GPUI cannot apply to element content directly.
//! - The "add to prompt" menu (beui's `MorphPopover`, `side="top" align="start"`) opens above the Plus
//!   button: `w-56 p-1.5` items (`rounded-lg px-2.5 py-2`, icon + label + muted description). GPUI has no
//!   `clip-path`, so the corner-morph grow is simplified to the fade-in this codebase already uses for
//!   `Disclosure`'s reveal; it still fades in on `card` with beui's drop-shadow, and closes at once like
//!   `Disclosure` does, rather than tweening the exit.
//! - Dictation (`set_dictation(true)`): a round microphone sits before Send. The first press asks the owner to start
//!   ([`PromptInputEvent::DictationStart`]); if the speech model is missing the owner shows the setup
//!   ([`PromptInput::set_voice_setup`]) and the left of the action row (Plus, model, mode) cross-fades into the setup
//!   bar. While it listens ([`PromptInput::set_voice_listening`]) the same place shows rounded amber bars that follow
//!   the voice ([`PromptInput::set_voice_level`]) with the time, and the microphone turns into a stop square on an
//!   amber disc. Stop gives [`PromptInputEvent::DictationStop`]; the owner then hands the words to
//!   [`PromptInput::insert_transcript`]. Send is off while it listens.
//! - Enter sends; Shift-Enter adds a line, exactly as before.

mod helpers;
mod impls;
mod structs;
mod types;

pub use helpers::append_transcript;
pub use structs::{PromptAction, PromptInput, PromptModel};
pub use types::PromptInputEvent;

#[cfg(test)]
use gpui_kit::{Bounds, Pixels};
#[cfg(test)]
use crate::scale::px;
#[cfg(test)]
use crate::{voice_input::VoiceMode, voice_setup::SetupPhase};

#[cfg(test)]
mod tests;
