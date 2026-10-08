use gpui_kit::{Bounds, Hsla, PathBuilder, Pixels, Window, point, px};

use super::types::STROKE;

/// The first `fraction` of a polyline, by length: the stroke drawing itself (`pathLength`).
pub fn prefix(points: &[(f32, f32)], fraction: f32) -> Vec<(f32, f32)> {
    let fraction = fraction.clamp(0., 1.);
    let length = |a: (f32, f32), b: (f32, f32)| ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
    let total: f32 = points.windows(2).map(|w| length(w[0], w[1])).sum();
    let mut left = total * fraction;
    let Some(&first) = points.first() else {
        return Vec::new();
    };
    let mut out = vec![first];
    for w in points.windows(2) {
        let run = length(w[0], w[1]);
        if left >= run {
            out.push(w[1]);
            left -= run;
        } else {
            let t = if run > 0. { left / run } else { 0. };
            out.push((
                w[0].0 + (w[1].0 - w[0].0) * t,
                w[0].1 + (w[1].1 - w[0].1) * t,
            ));
            break;
        }
    }
    out
}

/// Paints the first `fraction` of `points` in `color`, 1.5px wide at 12px, with round ends and joints.
pub(super) fn stroke(
    bounds: Bounds<Pixels>,
    points: &[(f32, f32)],
    fraction: f32,
    color: Hsla,
    window: &mut Window,
) {
    let drawn = prefix(points, fraction);
    if drawn.len() < 2 {
        return;
    }
    let unit = f32::from(bounds.size.width) / 24.;
    let at = |p: (f32, f32)| {
        point(
            bounds.origin.x + px(p.0 * unit),
            bounds.origin.y + px(p.1 * unit),
        )
    };
    let mut path = PathBuilder::stroke(px(STROKE * unit));
    path.move_to(at(drawn[0]));
    for p in &drawn[1..] {
        path.line_to(at(*p));
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
    // gpui strokes have butt ends: a dot at each end and joint makes them round.
    for p in &drawn {
        let mut dot = PathBuilder::fill();
        let (c, r) = (at(*p), STROKE * unit / 2.);
        for k in 0..16 {
            let a = k as f32 / 16. * std::f32::consts::TAU;
            let q = point(c.x + px(r * a.cos()), c.y + px(r * a.sin()));
            if k == 0 {
                dot.move_to(q)
            } else {
                dot.line_to(q)
            }
        }
        dot.close();
        if let Ok(dot) = dot.build() {
            window.paint_path(dot, color);
        }
    }
}
