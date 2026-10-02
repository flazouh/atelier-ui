//! The one-time setup of dictation: the first press of the microphone has to fetch the speech model, so the bar says so
//! in one quiet line: the words, a small segmented bar, and the size.
//!
//! SPIKE, with [`crate::voice_waves`]. The owner reports the work; this only draws it, with [`CellBar`](crate::cell_bar::CellBar), the one bar atelier
//! draws for anything that loads.
//!
//! - One row that hugs its content: the words in muted text, then the [`CellBar`](crate::cell_bar::CellBar), then the size (`69 / 164 MB`) in mono. It
//!   never stretches across the bar. The bar's color is [`amber_for`](crate::voice_waves::amber_for) the theme: deeper on a light page.
//! - Download: cells fill from the left in step with the number, which eases so a jumpy download does not step.
//! - Prepare: the model is on disk and loads, with no number to give. One cell steps along the row, a little slower than
//!   the eye follows, and that is the only motion.
//! - Ready: every cell is lit and the words turn to `Ready` with a check.
//! - Reduce Motion: the fill jumps to the number, and the loading cell rests in the middle.

mod helpers;
mod structs;
mod types;

pub use helpers::{copy, fraction};
pub use structs::VoiceSetup;
pub use types::SetupPhase;

#[cfg(test)]
mod tests;
