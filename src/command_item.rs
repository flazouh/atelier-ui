//! What the composer offers after `/` (commands) and `@` (files), as neutral data, and the rules for when
//! each list opens and what a pick writes. The composer ([`crate::prompt_input::PromptInput`]) draws the
//! list; the app fills it (the agent's commands, atelier's, the project's skills and files) and runs a pick.

mod helpers;
mod structs;
mod types;

pub use helpers::{ranked, trigger};
pub use structs::CommandItem;
pub use types::{CommandSource, Trigger};

#[cfg(test)]
mod tests;
