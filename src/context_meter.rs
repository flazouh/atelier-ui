//! How full the agent's context window is: a small ring that fills clockwise from the top, in the
//! composer's toolbar. Muted while there is room, amber from 80%, red from 95%. A hover tells the
//! numbers: "84k of 200k tokens (42%)".

mod helpers;
mod structs;
mod types;

pub use helpers::{fraction, ink, level, summary, tokens};
pub use structs::ContextMeter;
pub use types::{FULL_AT, Level, WARN_AT};

#[cfg(test)]
mod tests;
