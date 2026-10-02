//! A pull request named inline, as a small pill: its state mark and `#3344`. Pressing it reports it.
//!
//! Hovering opens a card by it, after gpui-base's `HoverCard` delay: `owner/repo`, the state, the title,
//! and two small ghost buttons, Copy link and Open in browser. The card is the app's one popover look,
//! Select's menu: a `card` fill, no border, and [`popover_shadow`](crate::theme::popover_shadow). The buttons live in that card, over
//! the text, not in the
//! pill: space reserved in the pill would leave a gap in the prose at rest, and buttons that grew the
//! pill on hover would push the words after it. The card fades and rises 4px in on
//! `duration::REVEAL`.
//!
//! [`crate::agent_text::AgentText`] makes chips from agent text: [`link_prs`] rewrites each `#N` the app
//! knows into a link to `atelier-pr:N`, and [`PrChips`], a Markdown plugin, draws those links as chips.

mod helpers;
mod structs;
mod types;

pub use helpers::{chip_number, link_prs};
pub use structs::{PrChip, PrChips};
pub use types::PrOpenHandler;

#[cfg(test)]
use crate::pr::PrChipData;

#[cfg(test)]
mod tests;
