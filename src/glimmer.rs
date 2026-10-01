//! Glimmer and band math for [`crate::thinking::Thinking`]'s label: the Claude Code CLI's glyph
//! glimmer (2.1.283, `Uyt` and its spinner hook) and the smooth band that glides between clusters
//! instead of stepping whole ones. Split out of thinking.rs because it is pure math with no GPUI in
//! it, so it stays small and easy to test on its own.
//!
//! Everything here counts and colors per grapheme *cluster*, not per Rust `char`. A decomposed accent
//! (a base letter followed by a combining mark), a variation selector, or a ZWJ-joined emoji sequence
//! must light and color as one unit, the way the CLI's own grapheme-aware width count sees it. beui
//! adds no segmentation dependency, so [`clusters`] groups these specific cases by hand from the
//! Unicode ranges the CLI actually needs; it is not a general grapheme-breaking algorithm.

use std::ops::Range;

use gpui_kit::{Hsla, HighlightStyle};

use crate::{motion::glimmer as tuning, theme::mix};

pub(crate) fn step_ms(requesting: bool) -> u64 {
    if requesting { tuning::REQUESTING_STEP_MS } else { tuning::STEP_MS }
}

fn cycle(text_width: i32) -> i32 {
    text_width + 2 * tuning::PAD
}

/// The CLI's `glimmerIndex`: the cluster at the glimmer's center, `elapsed_ms` after the row appeared.
/// Requesting walks left to right from `-pad`; every other mode walks right to left from
/// `text_width + pad`.
pub fn glimmer_index(elapsed_ms: u64, text_width: i32, requesting: bool) -> i32 {
    let step = (elapsed_ms / step_ms(requesting)) as i32;
    let pad = tuning::PAD;
    if requesting { step % cycle(text_width) - pad } else { text_width + pad - step % cycle(text_width) }
}

/// [`glimmer_index`] without the steps, for the smooth band. It equals the index on every step.
pub fn glimmer_center(elapsed_ms: u64, text_width: i32, requesting: bool) -> f32 {
    let steps = (elapsed_ms as f64 / step_ms(requesting) as f64) % cycle(text_width) as f64;
    let pad = tuning::PAD as f64;
    (if requesting { steps - pad } else { text_width as f64 + pad - steps }) as f32
}

/// How much the smooth band lights `cluster`: 1 at the center, 0 at [`tuning::BAND_REACH`] clusters.
pub fn glimmer_weight(cluster: usize, center: f32) -> f32 {
    (1. - (cluster as f32 - center).abs() / tuning::BAND_REACH).max(0.)
}

/// The CLI lights the cluster at the index and the one on each side.
pub fn stepped_lit(cluster: usize, index: i32) -> bool {
    (cluster as i32 - index).abs() <= 1
}

/// Milliseconds until the smooth band next touches the text, or 0 while it does.
pub fn band_wait_ms(elapsed_ms: u64, text_width: i32, requesting: bool) -> u64 {
    let (pad, width, reach) = (tuning::PAD as f32, text_width as f32, tuning::BAND_REACH);
    // Steps into the cycle at which the center comes within reach of the first and last cluster.
    let (enter, leave) = if requesting {
        (pad - reach, width - 1. + reach + pad)
    } else {
        (pad - (reach - 1.), width + pad + reach)
    };
    let step = step_ms(requesting) as f32;
    let cycle_ms = cycle(text_width) as u64 * step_ms(requesting);
    let at = (elapsed_ms % cycle_ms) as f32;
    let (enter, leave) = (enter * step, leave * step);
    if at < enter {
        (enter - at).ceil() as u64
    } else if at < leave {
        0
    } else {
        (cycle_ms as f32 - at + enter).ceil() as u64
    }
}

/// Combining marks that decorate the character before them: accents, Cyrillic and Latin extended
/// combining diacritics, and the two combining-mark blocks used for emphasis marks.
fn is_combining_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F)
}

/// `U+FE0E` (text) and `U+FE0F` (emoji) variation selectors: they pick a glyph style for the
/// character before them, so they are never a cluster of their own.
fn is_variation_selector(c: char) -> bool {
    matches!(c, '\u{FE0E}' | '\u{FE0F}')
}

/// Zero Width Joiner: glues the character after it to the one before, as in a family or flag emoji
/// sequence.
const ZWJ: char = '\u{200D}';

/// Groups `text` into clusters: a base character plus any combining marks, variation selectors, and
/// ZWJ-joined characters that follow it count as one cluster, so a decomposed accent or a ZWJ emoji
/// sequence lights and colors as a single unit instead of splitting across several `char`s.
pub fn clusters(text: &str) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Vec::new();
    let mut join_next = false;
    for (at, c) in text.char_indices() {
        let end = at + c.len_utf8();
        let attach = is_combining_mark(c) || is_variation_selector(c) || c == ZWJ || join_next;
        join_next = c == ZWJ;
        match (attach, out.last_mut()) {
            (true, Some(last)) => last.end = end,
            _ => out.push(at..end),
        }
    }
    out
}

/// How many clusters wide `text` is, as the glimmer's index math sees it.
pub fn cluster_count(text: &str) -> i32 {
    clusters(text).len() as i32
}

/// One color per cluster, as highlights for a single text run: `message` where `weight` is 0, `glimmer`
/// where it is 1.
pub fn glimmer_highlights(
    text: &str,
    message: Hsla,
    glimmer: Hsla,
    weight: impl Fn(usize) -> f32,
) -> Vec<(Range<usize>, HighlightStyle)> {
    clusters(text)
        .into_iter()
        .enumerate()
        .map(|(i, range)| {
            let color = mix(message, glimmer, weight(i).clamp(0., 1.));
            (range, HighlightStyle { color: Some(color), ..Default::default() })
        })
        .collect()
}

#[cfg(test)]
mod tests;
