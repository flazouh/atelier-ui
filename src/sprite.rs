//! Plays an animated mark from a sprite strip: an SVG that stacks its frames top to bottom.
//!
//! A [`Strip`] is plain data: where the SVG is served, its bytes, how many frames it holds, how long each
//! shows, and whether it loops. beui ships no strips of its own; an agent's strips reach it as data (see
//! [`crate::agent_look::AgentLook`]). Whoever owns the strips also serves them to GPUI under
//! [`Strip::path`], since [`Sprite`] draws with `svg().path()`.
//!
//! The strip plays as CSS `steps(frames, jump-none)`: each frame shows for the same time. GPUI has no
//! element transform, so [`Sprite`] clips a `size x size` box over the strip and moves the strip up by
//! whole frames.
//!
//! It wakes the view once per frame period with a timer ([`Wake`]), not every display frame. Under Reduce
//! Motion, or when it is not playing, it draws the first frame of its rest strip and stands still.
//!
//! Looping strips read a process-wide clock ([`shared_clock`]) instead of when that particular sprite
//! mounted, so every sprite showing the same looping strip lands on the same frame at the same moment and
//! they wake for the same redraw instead of drifting apart. One-shots keep their own start, since each
//! plays its own run once from the moment it is triggered.

use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

use gpui_kit::{App, ElementId, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window, div, svg};
use crate::scale::px;

use crate::wake::Wake;

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

/// The `(width, height)` in an SVG's `viewBox`. Some strips are exported a unit or two off an exact
/// `100 x frames*100`, so scaling the strip to a target width (as GPUI does, preserving the SVG's aspect
/// ratio) does not land each frame on a whole multiple of the requested size. Falls back to a square
/// 100-wide frame if `viewBox` is missing or malformed.
pub(crate) fn native_size(bytes: &[u8]) -> (f32, f32) {
    let fallback = (100., 100.);
    let text = std::str::from_utf8(bytes).unwrap_or_default();
    let Some(after) = text.split("viewBox=\"").nth(1) else { return fallback };
    let Some(value) = after.split('"').next() else { return fallback };
    let nums: Vec<f32> = value.split_whitespace().filter_map(|n| n.parse().ok()).collect();
    match nums.as_slice() {
        [_, _, w, h] => (*w, *h),
        _ => fallback,
    }
}

/// The pixel height of one frame when the strip is drawn `size` px wide, from its native
/// `(width, height)`. Equal to `size` when the viewBox is exactly `100 x frames*100`; a strip that is off
/// by a unit or two keeps its frames from drifting instead of assuming every frame is a perfect square.
pub(crate) fn frame_height(native: (f32, f32), frames: usize, size: f32) -> f32 {
    size * native.1 / native.0 / frames as f32
}

#[derive(IntoElement)]
pub struct Sprite {
    id: ElementId,
    strip: Strip,
    rest: Strip,
    size: Pixels,
    color: Hsla,
    playing: bool,
    still_frame: usize,
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

/// All sprites on a looping strip read from this same clock, so their frames stay in step with each
/// other no matter when each one mounted.
fn shared_clock() -> Instant {
    static START: OnceLock<Instant> = OnceLock::new();
    *START.get_or_init(Instant::now)
}

/// When the current one-shot started, and the wake-up for its next frame. Unused for looping strips,
/// which read [`shared_clock`] instead.
struct SpriteMotion {
    strip: Strip,
    start: Instant,
    wake: Wake,
    /// Whether the last render was still (reduce motion, or `.playing(false)`).
    still: bool,
}

impl SpriteMotion {
    fn new(strip: Strip) -> Self {
        Self { strip, start: Instant::now(), wake: Wake::default(), still: false }
    }
}

/// Whether the animation should restart at frame 0: on a changed strip, or when play resumes after being
/// still. A still sprite never advances `start`, so without this a one-shot would jump to its last frame
/// and a loop would resume mid-cycle instead of starting over.
pub(crate) fn restarts(current: &Strip, current_still: bool, requested: &Strip) -> bool {
    current != requested || current_still
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

#[cfg(test)]
mod tests;
