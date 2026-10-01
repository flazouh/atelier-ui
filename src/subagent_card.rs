//! One subagent in the chat, where it started, in TodoList's card language.
//!
//! - Header `h-11 px-3.5`: the look's mark (orbiting while it runs, still once done), the agent's name, its
//!   task in muted text, a [`ModelBadge`], and on the right the elapsed time, or a check once done.
//! - Body line, under the name: "12 tool calls" and the live tool call with a turning [`Spinner`]. When the
//!   live call or the count changes, the old text leaves and the new enters with [`Morph`]. Once done it
//!   reads "Done in 38s" and "12 tool calls", parted by [`SEGMENT_GAP`] of space.
//! - Pressing the card opens its tool calls, as [`ToolCall`] rows, with [`Reveal`].

mod helpers;
mod structs;

pub use helpers::status_line;
pub use structs::SubagentCard;

#[cfg(test)]
mod tests;
