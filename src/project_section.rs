//! The header of a project in the sidebar: a fold arrow, a mark for where it lives (a folder for a
//! project on this machine, a host mark and the host's name for one over SSH), its name, how a
//! remote project is connected, a `+` to start a session in it and a `⋯` for its menu.
//!
//! A remote project that is connecting or reconnecting shows a spinner and the word; one that is offline
//! shows the danger mark, "Offline" and a Retry action. A connected one shows nothing extra.

mod helpers;
mod structs;
mod types;

pub use crate::session_row::ROW_HEIGHT;

pub use helpers::{connection_words, unavailable};
pub use structs::ProjectSection;
pub use types::{MENU, MENU_ORIGIN, MenuChoice};

#[cfg(test)]
mod tests;
