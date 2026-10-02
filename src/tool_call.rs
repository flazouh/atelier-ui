//! beui's ToolResult (`components/agents/tool-result.tsx`), class for class:
//!
//! - Header `min-h-8 gap-2 rounded-md py-1 text-sm`: a `size-3.5` icon for the kind of call (a terminal, a file, a
//!   search), the title `font-medium text-foreground/90`, an optional meta `text-xs` at 60% muted, the call's
//!   argument in mono `text-[11px]` at 55% muted (the command, the path, the pattern), then the status as an icon
//!   alone, never in words: a muted spinner while it runs, one solid disc from `crate::theme::ramp` with its glyph
//!   knocked out in the page color when it is done, failed or cancelled; and a `size-3.5` chevron.
//! - Body `pl-6 pt-1.5`: a `rounded-xl` card holding the output in mono at `p-3`, capped at 220px, and a
//!   footer row with Copy and the status label.
//! - A card, as in [`crate::subagent_card::SubagentCard`]: `bg-card rounded-2xl`, its header as compact as a strip row (`h-8 px-3`), the output
//!   in a darker well inside it. [`ToolCall::flat`] drops the card and tightens the row: reading and searching are
//!   flat, so a run of them stacks close, and so is a call that sits inside another card, as in an open subagent.
//! - It opens while running and closes by itself when the tool finishes, like `collapseOnComplete`.

mod helpers;
mod structs;
mod types;

pub use structs::ToolCall;
pub use types::ToolStatus;
pub(crate) use types::{CARD_HEADER_HEIGHT, CARD_HEADER_PAD_X};

#[cfg(test)]
use helpers::should_open;

#[cfg(test)]
mod tests;
