//! The subagents of a running turn, one [`SubagentRow`](crate::subagent_row::SubagentRow) each, above the composer.
//!
//! The strip grows and shrinks as rows join and leave, never in a jump: each row's slot opens from no
//! height to its own on [`Spring::LAYOUT`](crate::motion::Spring::LAYOUT) and fades in with it, and closes the same way. A finished row
//! keeps its check for `duration::FINISH_HOLD`, then leaves, and stays gone while the data still
//! holds it finished; it joins again only if it runs again. A row that drops out of the data leaves
//! at once, drawn from its last known state. Rows already there on the first paint show at once. Under
//! Reduce Motion slots open and close at once; the hold stays, since it is timing, not movement.

mod helpers;
mod structs;
mod types;

pub use structs::SubagentStrip;
#[cfg(test)]
pub(crate) use structs::StripState;
pub use types::STACK_GAP;

#[cfg(test)]
use crate::{motion::duration, subagent_row::SubagentRow};

#[cfg(test)]
mod tests;
