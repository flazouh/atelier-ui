//! A pull request named inline, as a small pill: its state mark, `#3344`, the title without its
//! conventional-commit head cut to fit, and `+120 −34` once the forge said. Pressing it reports it.
//!
//! Hovering opens a card by it after 120 ms, and at once while another card is open or has just closed, so
//! the pointer can go from chip to chip. The card is a fixed 340px: `owner/repo #N` and the state, the whole
//! title, the author, the age and the comments, the checks, the review and the size bar, then Open, Copy link
//! and Open in browser. It is the app's one popover look, Select's menu: a `card` fill, no border, and
//! [`popover_shadow`](crate::theme::popover_shadow). Only the first card fades and rises 2px in, on
//! `duration::REVEAL`; one that takes over from another swaps in place.
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
