//! Words that arrive while someone is still talking. The engine hands over the whole sentence heard so far every half
//! second, and rewrites its last few words as it hears more. Each new word fades in, and a word the engine rewrote dips and
//! fades back in, so a change reads as the engine settling a word and not as the text flickering. No colour is added: a word
//! is only ever the text's own ink, at some strength.
//!
//! The model knows nothing of GPUI; time is handed in, so every case is a plain test.

mod structs;

pub use structs::{Ink, Word};

#[cfg(test)]
mod tests;
