//! ContextUsage: the panel that opens from the context ring. A header with the share and the numbers, one bar cut into
//! the parts that fill the window (the system prompt, the tools, the conversation), and under it a row for each part.
//!
//! It takes plain data: the tokens in use, the window, and the parts the agent could tell. It paints with the theme
//! and is a dropdown panel, so it has the one subtle 1px border (`AGENTS.md`).

mod helpers;
mod structs;
mod types;

pub use helpers::{header, height, precise, shares, swatch, totals};
pub use structs::ContextUsage;
pub use types::{ContextPart, WIDTH};

#[cfg(test)]
mod tests;
