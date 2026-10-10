use crate::motion::{cubic_bezier, ease};

use super::consts::{ACTION_SECONDS, ACTION_WAIT, WORD_STEP, WORDS_AT};

/// Where each word of `text` starts, after the first.
fn word_starts(text: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut in_word = false;
    for (i, c) in text.char_indices() {
        let space = c.is_whitespace();
        if !space && !in_word && i > 0 {
            starts.push(i);
        }
        in_word = !space;
    }
    starts
}

/// How many bytes of `text` are in at `elapsed`: whole words, one more every `WORD_STEP` from `WORDS_AT` on.
/// Under Reduce Motion the whole text is in from the start.
pub(super) fn shown(text: &str, elapsed: f32, reduce: bool) -> usize {
    if reduce || text.is_empty() {
        return text.len();
    }
    if elapsed < WORDS_AT {
        return 0;
    }
    let words = ((elapsed - WORDS_AT) / WORD_STEP).floor() as usize;
    word_starts(text).get(words).copied().unwrap_or(text.len())
}

/// When the last word of `text` is in.
pub(super) fn words_done(text: &str) -> f32 {
    WORDS_AT + WORD_STEP * word_starts(text).len() as f32
}

/// How far in the button is at `elapsed`: it comes `ACTION_WAIT` after the last word, over `ACTION_SECONDS`.
pub(super) fn action(text: &str, elapsed: f32, reduce: bool) -> f32 {
    if reduce {
        return 1.;
    }
    let t = (elapsed - words_done(text) - ACTION_WAIT) / ACTION_SECONDS;
    if t <= 0. {
        0.
    } else if t >= 1. {
        1.
    } else {
        cubic_bezier(ease::OUT, t)
    }
}

/// Whether anything still moves at `elapsed`, so the page asks for another frame.
pub(super) fn moving(text: &str, elapsed: f32, reduce: bool) -> bool {
    !reduce && elapsed < words_done(text) + ACTION_WAIT + ACTION_SECONDS
}
