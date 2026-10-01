//! Spinner: the `spinner` variant of beui.dev's Loader (`components/motion/loader.tsx`). A ring drawn at
//! 20% with a quarter arc on it, round at both ends, turning once a second at a steady speed. The stroke is
//! 9% of the size and never under 2px. Under Reduce Motion nothing turns: the whole ring pulses from full
//! to 40% and back over 1.4s.
//!
//! The other sixteen variants of the Loader (dots, bars, dot-matrix, dither, the ascii sets, morph, comet,
//! scramble, metaballs, newton, helix, percent) are not built; atelier shows only work in progress.
use std::f32::consts::{FRAC_PI_2, TAU};

use gpui_kit::{px, App, Bounds, ElementId, Hsla, IntoElement, Pixels, RenderOnce, Window, canvas, div, point, PathBuilder, prelude::FluentBuilder, Styled, ParentElement};

use crate::{
    motion::{keyframes, now_millis},
    theme::ActiveTheme,
};

/// How long one turn takes.
pub const TURN_MILLIS: u128 = 1000;
/// How long the reduced-motion pulse takes, and its lowest opacity.
pub const PULSE_MILLIS: u128 = 1400;
pub const PULSE_LOW: f32 = 0.4;
/// How strong the ring under the arc is.
pub const RING_ALPHA: f32 = 0.2;
/// The arc's length: a quarter of the ring.
const SWEEP: f32 = FRAC_PI_2;
/// The steps a ring is drawn in.
const STEPS: usize = 48;

/// The stroke for a spinner of `size`: 9% of it, at least 2px.
pub fn stroke_width(size: f32) -> f32 {
    (size * 0.09).max(2.)
}

/// The angle the arc starts at, in radians from the top, `millis` into the animation.
pub fn turn_at(millis: u128) -> f32 {
    (millis % TURN_MILLIS) as f32 / TURN_MILLIS as f32 * TAU
}

/// The opacity of the whole ring `millis` into the reduced-motion pulse.
pub fn pulse_at(millis: u128) -> f32 {
    let t = (millis % PULSE_MILLIS) as f32 / PULSE_MILLIS as f32;
    keyframes(&[1., PULSE_LOW, 1.], &[0., 0.5, 1.], 1., [0.42, 0., 0.58, 1.], t)
}

/// Points on a circle of `radius` round `centre`, from `start` (radians from the top, clockwise) over `sweep`.
pub fn arc(centre: (f32, f32), radius: f32, start: f32, sweep: f32, steps: usize) -> Vec<(f32, f32)> {
    (0..=steps)
        .map(|i| {
            let a = start + sweep * i as f32 / steps as f32;
            (centre.0 + radius * a.sin(), centre.1 - radius * a.cos())
        })
        .collect()
}

#[derive(IntoElement)]
pub struct Spinner {
    id: ElementId,
    size: Pixels,
    color: Option<Hsla>,
}

impl Spinner {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), size: px(14.), color: None }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// Defaults to the foreground.
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

fn stroke(points: &[(f32, f32)], width: f32, bounds: Bounds<Pixels>, color: Hsla, window: &mut Window) {
    let at = |p: (f32, f32)| point(bounds.origin.x + px(p.0), bounds.origin.y + px(p.1));
    let mut path = PathBuilder::stroke(px(width));
    let mut it = points.iter();
    if let Some(first) = it.next() {
        path.move_to(at(*first));
    }
    for p in it {
        path.line_to(at(*p));
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
}

/// A filled dot, to round an end of the arc (gpui's strokes have butt ends).
fn dot(centre: (f32, f32), radius: f32, bounds: Bounds<Pixels>, color: Hsla, window: &mut Window) {
    let mut path = PathBuilder::fill();
    for k in 0..16 {
        let a = k as f32 / 16. * TAU;
        let q = point(bounds.origin.x + px(centre.0 + radius * a.cos()), bounds.origin.y + px(centre.1 + radius * a.sin()));
        if k == 0 { path.move_to(q) } else { path.line_to(q) }
    }
    path.close();
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
}

impl RenderOnce for Spinner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self.color.unwrap_or(cx.theme().foreground);
        let reduce = cx.reduce_motion();
        let size = f32::from(self.size);
        let millis = now_millis();
        let (start, opacity) = if reduce { (0., pulse_at(millis)) } else { (turn_at(millis), 1.) };
        // It draws again every frame while it is on screen.
        window.request_animation_frame();
        let _ = &self.id;
        div().flex_none().size(self.size).when(reduce, |d| d.opacity(opacity)).child(canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let width = stroke_width(size);
                let r = (size - width) / 2.;
                let c = (size / 2., size / 2.);
                stroke(&arc(c, r, 0., TAU, STEPS), width, bounds, color.opacity(RING_ALPHA), window);
                let arc_points = arc(c, r, start, SWEEP, STEPS / 4);
                stroke(&arc_points, width, bounds, color, window);
                for end in [arc_points.first(), arc_points.last()].into_iter().flatten() {
                    dot(*end, width / 2., bounds, color, window);
                }
            },
        ).size_full())
    }
}

#[cfg(test)]
mod tests;
