//! Glimmer and band math for [`crate::thinking::Thinking`]'s label: the Claude Code CLI's glyph
//! glimmer (2.1.283, `Uyt` and its spinner hook) and the smooth band that glides between clusters
//! instead of stepping whole ones. Split out of thinking.rs because it is pure math with no GPUI in
//! it, so it stays small and easy to test on its own.
//!
//! Everything here counts and colors per grapheme *cluster*, not per Rust `char`. A decomposed accent
//! (a base letter followed by a combining mark), a variation selector, or a ZWJ-joined emoji sequence
//! must light and color as one unit, the way the CLI's own grapheme-aware width count sees it. beui
//! adds no segmentation dependency, so `clusters` groups these specific cases by hand from the
//! Unicode ranges the CLI actually needs; it is not a general grapheme-breaking algorithm.

mod helpers;
mod types;

#[cfg(test)]
pub use helpers::clusters;
pub(crate) use helpers::step_ms;
pub use helpers::{
    band_wait_ms, cluster_count, cursor_weight, glimmer_center, glimmer_highlights, glimmer_index,
    glimmer_weight, stepped_lit,
};

#[cfg(test)]
mod tests;
