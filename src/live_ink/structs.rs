use std::{ops::Range, time::Instant};

/// How fast a word comes to full ink: the time it takes to cover 63% of the way. About 150 ms for it to be whole.
const FADE: f32 = 0.055;
/// How far a word the engine rewrote falls back to, at most.
const DIP: f32 = 0.3;

/// One word of the text, and how much of its ink shows, 0 to 1.
#[derive(Clone, Debug, PartialEq)]
pub struct Word {
    pub range: Range<usize>,
    pub alpha: f32,
}

#[derive(Clone, Debug)]
pub struct Ink {
    text: String,
    words: Vec<Word>,
    last: Instant,
    still: bool,
}

fn split(text: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut start = None;
    for (i, c) in text.char_indices() {
        match (c.is_whitespace(), start) {
            (false, None) => start = Some(i),
            (true, Some(s)) => {
                out.push(s..i);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        out.push(s..text.len());
    }
    out
}

impl Ink {
    pub fn new(now: Instant) -> Self {
        Self { text: String::new(), words: Vec::new(), last: now, still: false }
    }

    /// Reduced motion: words show whole the moment they come, and nothing is animated.
    pub fn still(mut self) -> Self {
        self.still = true;
        self
    }

    pub fn set_still(&mut self, still: bool) {
        self.still = still;
    }

    pub fn words(&self) -> &[Word] {
        &self.words
    }

    /// The engine now says `text`. A word at a place that held another word was rewritten; a place past the old end is new.
    pub fn observe(&mut self, text: &str) {
        let words = split(text)
            .into_iter()
            .enumerate()
            .map(|(i, range)| {
                let alpha = match self.words.get(i) {
                    _ if self.still => 1.,
                    Some(old) if self.text[old.range.clone()] == text[range.clone()] => old.alpha,
                    Some(old) => old.alpha.min(DIP),
                    None => 0.,
                };
                Word { range, alpha }
            })
            .collect();
        self.words = words;
        self.text = text.to_string();
    }

    /// Moves the words on to `now`.
    pub fn step(&mut self, now: Instant) {
        let dt = now.saturating_duration_since(self.last).as_secs_f32();
        self.last = now;
        let k = 1. - (-dt / FADE).exp();
        for w in &mut self.words {
            w.alpha += (1. - w.alpha) * k;
            if w.alpha > 0.995 {
                w.alpha = 1.;
            }
        }
    }

    /// Whether a word is still coming to full ink, so a frame is worth asking for.
    pub fn moving(&self) -> bool {
        self.words.iter().any(|w| w.alpha < 1.)
    }
}
