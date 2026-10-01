//! The answer as it streams in. A finished paragraph is Markdown (the `TextView` as always); the paragraph still
//! growing, the tail, is drawn as plain runs whose ink follows their age, so each new piece fades in over 240ms.
//! Only the newest pieces fade, and once the stream stalls no frame is asked for. A tail that needs Markdown (a
//! list, a heading, a code span, a link) is left to `TextView`, with no fade.
//!
//! The tail sits under the same 12px gap a paragraph has, in the same text style, so when it ends and joins the
//! Markdown nothing moves (`tests.rs` lays both out and compares). See plans/agent-activity.md.
use std::{ops::Range, time::{Duration, Instant}};

/// How long a piece takes to reach full ink.
pub const FADE: Duration = Duration::from_millis(240);

/// The byte where the tail begins: after the last blank line, unless that line is inside a code fence, where the
/// whole text is Markdown and the answer is `text.len()`.
pub fn split_tail(text: &str) -> usize {
    let mut fence = false;
    let mut last = 0;
    let mut at = 0;
    let mut blank_run = false;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fence = !fence;
        }
        at += line.len();
        if trimmed.is_empty() && !fence {
            blank_run = true;
        } else if blank_run {
            // `last` is where this first line of the next paragraph began.
            last = at - line.len();
            blank_run = false;
        }
    }
    if fence {
        return text.len();
    }
    if blank_run {
        return text.len();
    }
    last
}

/// Whether `tail` is text that reads the same with or without Markdown: one paragraph of words.
pub fn is_plain(tail: &str) -> bool {
    let first = tail.trim_start_matches(' ');
    if tail.starts_with("    ") || tail.contains('\n') {
        return false;
    }
    let starts_block = first.starts_with(['#', '>', '|'])
        || first.starts_with("- ")
        || first.starts_with("* ")
        || first.starts_with("+ ")
        || first.starts_with("```")
        || first.starts_with("~~~")
        || first.starts_with("---")
        || first.split_once(['.', ')']).is_some_and(|(n, rest)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) && rest.starts_with(' '));
    !starts_block && !tail.contains(['`', '*', '_', '[', '<', '\\', '!', '~'])
}

/// When each piece of the growing text arrived: the start of the piece, and the moment.
#[derive(Clone, Debug, Default)]
pub struct Flow {
    seen: usize,
    marks: Vec<(usize, Instant)>,
}
impl Flow {
    /// Notes that the text is now `text`: a longer text is a new piece from where the last ended, a shorter one is a
    /// new text.
    pub fn observe(&mut self, text: &str, now: Instant) {
        if text.len() < self.seen {
            self.marks.clear();
            self.seen = 0;
        }
        if text.len() > self.seen {
            self.marks.push((self.seen, now));
            self.seen = text.len();
        }
        self.marks.retain(|(_, at)| now.saturating_duration_since(*at) < FADE);
    }
    /// Pieces still coming in.
    pub fn marks(&self) -> usize {
        self.marks.len()
    }
    /// Whether a piece is still fading, so a frame is worth asking for.
    pub fn is_fading(&self, now: Instant) -> bool {
        self.marks.iter().any(|(_, at)| now.saturating_duration_since(*at) < FADE)
    }
    /// The pieces of `text` from `tail` on that have not reached full ink, each with its ink, 0 to 1. Text older than
    /// the fade has no entry: it is full ink as it stands.
    pub fn alphas(&self, text: &str, tail: usize, now: Instant) -> Vec<Piece> {
        let mut out = Vec::new();
        for (i, (start, at)) in self.marks.iter().enumerate() {
            let age = now.saturating_duration_since(*at);
            if age >= FADE {
                continue;
            }
            let end = self.marks.get(i + 1).map_or(text.len(), |(next, _)| *next).min(text.len());
            let start = (*start).max(tail);
            if start >= end {
                continue;
            }
            let t = age.as_secs_f32() / FADE.as_secs_f32();
            out.push((start..end, ease_out(t)));
        }
        out
    }
}
/// A piece of the tail and its ink, as the view draws it: the bytes, and 0 to 1.
pub type Piece = (Range<usize>, f32);
/// Motion's default ease-out, `[0, 0, 0.58, 1]`, which the rest of the app uses for a plain fade.
fn ease_out(t: f32) -> f32 {
    crate::motion::cubic_bezier(crate::motion::ease::STANDARD_MOTION, t.clamp(0., 1.))
}
#[cfg(test)]
mod tests;
