//! How an agent looks in the agent panel: its animated mark, the colours of its status label, and the
//! words for each phase. atelier-ui knows no agent; an agent crate builds an [`AgentLook`] and hands it to
//! [`crate::thinking::Thinking`] and [`crate::subagent_row::SubagentRow`] as data.
//!
//! [`AgentLook::neutral`] is a plain fallback for tests and for stories that show no particular agent.

mod structs;
mod types;

pub use structs::{AgentLook, Mark, PhaseLabels};

#[cfg(test)]
mod tests;
