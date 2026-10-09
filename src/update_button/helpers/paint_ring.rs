use gpui_kit::{Bounds, Hsla, Pixels, Window};

use crate::{
    scale::px,
    spinner::{dot, stroke},
    update_button::{consts::RING_STROKE, helpers::ring_points},
};

/// Paints the track and then the arc (`fraction` of a turn from `start`) with round ends, inside `bounds`.
pub(in crate::update_button) fn paint_ring(
    fraction: f32,
    start: f32,
    track: Hsla,
    arc: Hsla,
    bounds: Bounds<Pixels>,
    window: &mut Window,
) {
    let scale = f32::from(px(1.));
    let scaled = |points: Vec<(f32, f32)>| points.into_iter().map(|(x, y)| (x * scale, y * scale)).collect::<Vec<_>>();
    let width = RING_STROKE * scale;
    stroke(&scaled(ring_points(1., 0.)), width, bounds, track, window);
    let points = scaled(ring_points(fraction, start));
    stroke(&points, width, bounds, arc, window);
    for end in [points.first(), points.last()].into_iter().flatten() {
        dot(*end, width / 2., bounds, arc, window);
    }
}
