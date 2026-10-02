//! The answer as it streams in. A finished paragraph is Markdown (the `TextView` as always); the paragraph still
//! growing, the tail, is drawn as plain runs whose ink follows their age, so each new piece fades in over 240ms.
//! Only the newest pieces fade, and once the stream stalls no frame is asked for. A tail that needs Markdown (a
//! list, a heading, a code span, a link) is left to `TextView`, with no fade.
//!
//! The tail sits under the same 12px gap a paragraph has, in the same text style, so when it ends and joins the
//! Markdown nothing moves (`tests.rs` lays both out and compares). See plans/agent-activity.md.

mod helpers;
mod structs;
mod types;

pub use helpers::{is_plain, split_tail};
pub use structs::Flow;
pub use types::{FADE, Piece};

#[cfg(test)]
mod tests;
