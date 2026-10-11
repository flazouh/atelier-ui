use std::time::Instant;

use gpui_kit::{App, ElementId, IntoElement, RenderOnce, SharedString, Window};

use super::types::{FADE, Piece};
use super::helpers::ease_in_out;
use crate::{glyph_text::GlyphText, motion};

/// Text that streams in: each piece that arrives fades in over [`FADE`], and what has not arrived keeps its place
/// with no ink, so nothing moves as the text fills. It is the one place that fade is drawn: the answer's growing
/// paragraph uses it, and so does any other text that comes in the same way.
///
/// - By default all of `text` has arrived, and a longer text on a later frame is a new piece, as a stream gives it.
/// - [`Self::shown`] says how many bytes have arrived, for an owner that holds the whole text and lets it in a
///   piece at a time.
/// - It fades by the ink's own strength, so it reads on any surface, a picture too.
/// - Under Reduce Motion a piece shows at once.
#[derive(IntoElement)]
pub struct Streamed {
    id: ElementId,
    text: SharedString,
    shown: Option<usize>,
}

impl Streamed {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self { id: id.into(), text: text.into(), shown: None }
    }
    /// How many bytes of the text have arrived. It must fall between two characters.
    pub fn shown(mut self, bytes: usize) -> Self {
        self.shown = Some(bytes);
        self
    }
}

impl RenderOnce for Streamed {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let shown = self.shown.unwrap_or(self.text.len()).min(self.text.len());
        let flow = window.use_keyed_state(self.id, cx, |_, _| Flow::default());
        let now = motion::now();
        let (mut fades, fading) = flow.update(cx, |flow, _| {
            flow.observe(&self.text[..shown], now);
            (flow.alphas(&self.text[..shown], 0, now), flow.is_fading(now))
        });
        if cx.reduce_motion() {
            fades.clear();
        } else if fading {
            window.request_animation_frame();
        }
        if shown < self.text.len() {
            fades.push((shown..self.text.len(), 0.));
        }
        GlyphText::new(self.text).fades(fades)
    }
}

/// When each piece of the growing text arrived: the start of the piece, and the moment.
#[derive(Clone, Debug, Default)]
pub struct Flow {
    seen: usize,
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
            out.push((start..end, ease_in_out(t)));
        }
        out
    }
}
