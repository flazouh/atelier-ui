use crate::motion::{cubic_bezier, ease};

use super::consts::{ACTION_SECONDS, ACTION_WAIT, PICTURE_SECONDS, TEXT_STEP, TEXT_WAIT, TITLE_AT, TITLE_STEP};

/// How far a part that starts at `at` and takes `seconds` has come at `elapsed`, eased out: exactly 0 before it
/// starts and exactly 1 once it is in.
fn progress(elapsed: f32, at: f32, seconds: f32) -> f32 {
    let t = (elapsed - at) / seconds;
    if t <= 0. {
        0.
    } else if t >= 1. {
        1.
    } else {
        cubic_bezier(ease::OUT, t)
    }
}

/// How far in the picture is. `elapsed` is None while the picture is not loaded: then nothing of it shows.
pub(super) fn picture(elapsed: Option<f32>, reduce: bool) -> f32 {
    match elapsed {
        None => 0.,
        Some(_) if reduce => 1.,
        Some(elapsed) => progress(elapsed, 0., PICTURE_SECONDS),
    }
}

/// When a text starts to come in, and how long between two of its words.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Pace {
    pub at: f32,
    pub step: f32,
}

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

/// How many bytes of `text` are in at `elapsed`: whole words, one more every step from the pace's start on.
/// Under Reduce Motion the whole text is in from the start.
pub(super) fn shown(text: &str, pace: Pace, elapsed: f32, reduce: bool) -> usize {
    if reduce || text.is_empty() {
        return text.len();
    }
    if elapsed < pace.at {
        return 0;
    }
    let words = ((elapsed - pace.at) / pace.step).floor() as usize;
    word_starts(text).get(words).copied().unwrap_or(text.len())
}

/// When the last word of `text` is in.
fn words_done(text: &str, pace: Pace) -> f32 {
    pace.at + pace.step * word_starts(text).len() as f32
}

/// The title's pace: it comes first.
pub(super) fn title_pace() -> Pace {
    Pace { at: TITLE_AT, step: TITLE_STEP }
}

/// The line's pace: it starts a moment after the title's last word, and its words come faster.
pub(super) fn text_pace(title: &str) -> Pace {
    Pace { at: words_done(title, title_pace()) + TEXT_WAIT, step: TEXT_STEP }
}

/// When the last word of the line is in.
pub(super) fn all_in(title: &str, text: &str) -> f32 {
    words_done(text, text_pace(title))
}

/// How far in the button is at `elapsed`: it comes `ACTION_WAIT` after the last word, over `ACTION_SECONDS`.
pub(super) fn action(title: &str, text: &str, elapsed: f32, reduce: bool) -> f32 {
    if reduce {
        return 1.;
    }
    progress(elapsed, all_in(title, text) + ACTION_WAIT, ACTION_SECONDS)
}

/// Whether anything still moves at `elapsed`, so the page asks for another frame.
pub(super) fn moving(title: &str, text: &str, elapsed: f32, reduce: bool) -> bool {
    !reduce && elapsed < all_in(title, text) + ACTION_WAIT + ACTION_SECONDS
}
