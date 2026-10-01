use std::{
    time::{Duration, Instant},
};

use gpui_kit::{
    App, ElementId, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window, div,
    svg,
};

use crate::scale::px;
use crate::wake::Wake;
use super::helpers::{frame_height, native_size, restarts, shared_clock};

/// One animation as a vertical strip of square frames.
#[derive(Clone, Copy)]
pub struct Strip {
    /// The asset path GPUI loads the SVG from.
    pub path: &'static str,
    /// The SVG itself, read for its `viewBox`.
    pub bytes: &'static [u8],
    pub frames: usize,
    pub frame_ms: u64,
    /// Looping strips play until they change; the others play once and hold their last frame.
    pub loops: bool,
}

impl PartialEq for Strip {
    /// Two strips are the same animation when they load the same asset. Comparing the bytes too would
    /// read tens of kilobytes on every render.
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.frames == other.frames && self.frame_ms == other.frame_ms && self.loops == other.loops
    }
}

impl Eq for Strip {}

impl std::fmt::Debug for Strip {
    /// Leaves out the bytes, which are a whole SVG.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Strip")
            .field("path", &self.path)
            .field("frames", &self.frames)
            .field("frame_ms", &self.frame_ms)
            .field("loops", &self.loops)
            .finish()
    }
}

impl Strip {
    /// The frame showing `elapsed_ms` after the strip started. Looping strips wrap; one-shots hold their
    /// last frame.
    pub fn frame_at(&self, elapsed_ms: u64) -> usize {
        let step = (elapsed_ms / self.frame_ms) as usize;
        if self.loops { step % self.frames } else { step.min(self.frames - 1) }
    }

    /// Milliseconds until the frame after the one at `elapsed_ms`, or `None` once a one-shot shows its
    /// last frame and nothing will move again.
    pub fn next_frame_in(&self, elapsed_ms: u64) -> Option<u64> {
        let period = self.frame_ms;
        if !self.loops && elapsed_ms / period >= self.frames as u64 - 1 {
            return None;
        }
        Some(period - elapsed_ms % period)
    }

    /// The strip's own `(width, height)`, from its `viewBox`.
    pub fn native_size(&self) -> (f32, f32) {
        native_size(self.bytes)
    }
}

#[derive(IntoElement)]
pub struct Sprite {
    pub(super) id: ElementId,
    pub(super) strip: Strip,
    pub(super) rest: Strip,
    pub(super) size: Pixels,
    pub(super) color: Hsla,
    pub(super) playing: bool,
    pub(super) still_frame: usize,
}

impl Sprite {
    /// Plays `strip` in `color`. It rests on the first frame of `strip` until [`Sprite::rest`] names
    /// another.
    pub fn new(id: impl Into<ElementId>, strip: Strip, color: impl Into<Hsla>) -> Self {
        Self { id: id.into(), strip, rest: strip, size: px(18.), color: color.into(), playing: true, still_frame: 0 }
    }

    /// The strip whose first frame shows while it stands still.
    pub fn rest(mut self, rest: Strip) -> Self {
        self.rest = rest;
        self
    }

    /// Defaults to 18.
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// The frame of the rest strip to show while it stands still. Defaults to the first.
    pub fn still_frame(mut self, frame: usize) -> Self {
        self.still_frame = frame;
        self
    }

    /// When false it shows the still mark, as when work has ended.
    pub fn playing(mut self, playing: bool) -> Self {
        self.playing = playing;
        self
    }
}

/// When the current one-shot started, and the wake-up for its next frame. Unused for looping strips,
/// which read [`shared_clock`] instead.
pub(super) struct SpriteMotion {
    pub(super) strip: Strip,
    pub(super) start: Instant,
    pub(super) wake: Wake,
    /// Whether the last render was still (reduce motion, or `.playing(false)`).
    pub(super) still: bool,
}

impl SpriteMotion {
    pub(super) fn new(strip: Strip) -> Self {
        Self { strip, start: Instant::now(), wake: Wake::default(), still: false }
    }
}

impl RenderOnce for Sprite {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let still = cx.reduce_motion() || !self.playing;
        let (strip, rest) = (self.strip, self.rest);
        let motion = window.use_keyed_state(self.id, cx, move |_, _| SpriteMotion::new(strip));
        let (shown, frame) = motion.update(cx, |m, cx| {
            if still {
                m.wake.cancel();
                m.still = true;
                return (rest, self.still_frame.min(rest.frames.saturating_sub(1)));
            }
            // A new strip, or resuming after being still, restarts a one-shot at frame 0. A loop does not
            // need its own restart: it always reads the shared clock, so it is already in step with every
            // other sprite on the same strip.
            if restarts(&m.strip, m.still, &strip) {
                *m = SpriteMotion::new(strip);
            }
            let elapsed = if strip.loops {
                shared_clock().elapsed().as_millis() as u64
            } else {
                m.start.elapsed().as_millis() as u64
            };
            match strip.next_frame_in(elapsed) {
                Some(wait) => m.wake.at(Instant::now() + Duration::from_millis(wait), cx),
                None => m.wake.cancel(),
            }
            (strip, strip.frame_at(elapsed))
        });
        // A whole number of pixels: at a fractional size (a zoom) the frames would not meet the clip and a sliver of the
        // next one would show.
        let size = gpui_kit::px(f32::from(self.size).round().max(1.));
        let frame_h = px(frame_height(shown.native_size(), shown.frames, size.into()));
        div().flex_none().w(size).h(frame_h).overflow_hidden().child(
            svg()
                .path(shown.path)
                .flex_none()
                .relative()
                .top(-frame_h * frame as f32)
                .w(size)
                .h(frame_h * shown.frames as f32)
                .text_color(self.color),
        )
    }
}
