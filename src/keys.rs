//! What the keyboard can ask for, and which keys ask for it: GitQuiet's table (`src/keys/commands.ts`),
//! ported with its words.
//!
//! One table, read by two things: the matcher that turns a press into a [`Command`], and the buttons that
//! wear a cap. The cap on a button comes out of the same table ([`chord_for`]), so the letter on a
//! control and the letter that works are the same letter by construction.
//!
//! Bare letters never type into text and are never taken from it: an owner reads a press with [`read`]
//! only while no text input has focus (see [`typing`]). The review's own keys, which GitQuiet has no
//! word for (accept or reject a hunk or a whole file), hold a modifier and stay GPUI key bindings.
//!
//! A chord is the key as the reader presses it: `s`, `T` (Shift and t), `/`, `Escape`, `⌘b` (Command or
//! Control and b), `⌘⇧b`. Two keys one after the other are one chord with a space, `g d`.

mod helpers;
mod structs;
mod types;

pub use helpers::{
    cap, cap_on, chord_for, chords, gist, held_down, profile, read, read_now, set_profile,
    typing, word,
};
pub use structs::{Keys, Press};
pub use types::{Command, KEYBOARD, PATIENCE, Profile, Waiting};

#[cfg(test)]
use std::time::Instant;

#[cfg(test)]
mod tests;
