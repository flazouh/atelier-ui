use std::f32::consts::TAU;

use gpui_kit::{Bounds, Hsla, PathBuilder, Pixels, Window, point, px};

use crate::motion::keyframes;
use super::types::{PULSE_LOW, PULSE_MILLIS, TURN_MILLIS};

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

pub(crate) fn stroke(points: &[(f32, f32)], width: f32, bounds: Bounds<Pixels>, color: Hsla, window: &mut Window) {
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
pub(crate) fn dot(centre: (f32, f32), radius: f32, bounds: Bounds<Pixels>, color: Hsla, window: &mut Window) {
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
