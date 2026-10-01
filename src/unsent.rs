//! Unsent Comments: comments the reader wrote on lines that are held, and shown to nobody else, until the
//! review carrying them is sent. Their count sits beside the control that sends them. The words never say
//! "pending", which readers take to mean somebody else owes a reply, nor "draft", which is already a pull
//! request's own state (GitQuiet's `CONTEXT.md`).

mod helpers;
mod structs;
mod types;

pub use helpers::{count_text, send_text};
pub use structs::UnsentComments;
pub use types::WHO_SEES;

#[cfg(test)]
use gpui_kit::SharedString;

#[cfg(test)]
mod tests;
