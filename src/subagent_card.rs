//! One subagent in the chat, where it started, in TodoList's card language.
//!
//! - Header `h-11 px-3.5`: the look's mark (orbiting while it runs, still once done), the agent's name, its
//!   task in muted text, a [`ModelBadge`](crate::model_badge::ModelBadge), and on the right the elapsed time, or a check once done.
//! - Body line, under the name: the live tool call flush left, under the mark rather than the name, and "12 tool calls" on the far right. The mark in
//!   the header already shows that it runs, so the line has no spinner of its own. Every kind of call (read,
//!   edit, search, web search) reads the same way here. When the live call or the count changes, the old text
//!   leaves and the new enters with [`Morph`](crate::morph::Morph). Once done the left side reads "Done in 38s".
//! - Pressing the card opens its tool calls, as [`ToolCall`](crate::tool_call::ToolCall) rows, with `Reveal`.

mod helpers;
mod structs;

pub use helpers::lead_text;
pub use structs::SubagentCard;

#[cfg(test)]
mod tests;
