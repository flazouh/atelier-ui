//! The status row shown while the agent works: the agent's mark, then the label and its segments. The
//! mark, the label's colours, and its words come from an [`AgentLook`](crate::agent_look::AgentLook).
//!
//! It follows the Claude desktop app's status line (`M5` in `c1c1ec7b9-BpMPRL5q.js`): the label follows
//! the phase and, while thinking, the elapsed seconds; every label swap runs [`Morph`](crate::morph::Morph); segments sit
//! after the label with a 12px gap, and no separator glyph. The text shimmer is the Claude Code CLI's glimmer (2.1.283, `Uyt` and its spinner
//! hook): the label in the look's message colour with three clusters in its glimmer colour walking across it, one per 200ms, then
//! leaving the text for a 10 cluster pad. [`Shimmer::Stepped`] is that exact look; [`Shimmer::Cursor`] is Cursor's band; [`Shimmer::Smooth`]
//! moves a soft band with the same center, speed, and colors. [`ThinkingStyle::Breath`] is the desktop
//! app's opacity breath instead. The glimmer and band math live in `crate::glimmer`.
//!
//! While subagents run, the mark orbits. Each subagent gets its own [`SubagentRow`](crate::subagent_row::SubagentRow), shown above the
//! composer.
//!
//! Under Reduce Motion nothing glimmers or breathes, labels swap at once, and the mark stands still.

mod helpers;
mod structs;
mod types;

pub use helpers::{label, mark_strip, next_label_change_s, tasks_text, tokens_text};
#[cfg(test)]
pub(crate) use helpers::{label_color, loading_strip, segment_text};
pub use structs::Thinking;
pub use types::{Shimmer, ThinkingPhase, ThinkingStyle};
#[cfg(test)]
pub(crate) use types::SEGMENT_GAP_TEXT;

#[cfg(test)]
use helpers::roll;

#[cfg(test)]
use gpui_kit::{Hsla, SharedString};
#[cfg(test)]
use crate::{
    agent_look::{AgentLook, Mark, PhaseLabels},
    glimmer,
};

#[cfg(test)]
mod tests;
