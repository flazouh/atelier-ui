//! One compact row per subagent, stacked in [`crate::subagent_strip::SubagentStrip`] above the composer
//! while a turn runs.
//!
//! A running row has the agent's orbiting mark, the agent's name in its message colour, its live tool
//! call in muted text (or its task, before the first call), its tool call count, and its elapsed time on
//! the right. When the live tool call changes, the old text leaves and the new one enters with [`Morph`].
//! A finished row stops its mark, shows a check, and says "Done in 38s".

mod helpers;
mod structs;
mod types;

pub use helpers::{done_text, tool_calls_text};
pub use structs::SubagentRow;
pub use types::ROW_HEIGHT;

#[cfg(test)]
use crate::agent_look::AgentLook;

#[cfg(test)]
mod tests;
