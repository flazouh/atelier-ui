use std::{
    time::{Instant},
};

use super::types::{FADE, Piece};
use super::helpers::ease_out;

/// When each piece of the growing text arrived: the start of the piece, and the moment.
#[derive(Clone, Debug, Default)]
pub struct Flow {
    pub(super) seen: usize,
    pub(super) marks: Vec<(usize, Instant)>,
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
