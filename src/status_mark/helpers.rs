use std::f32::consts::{FRAC_PI_2, TAU};

use gpui_kit::{Bounds, Hsla, PathBuilder, Pixels, Point, Window, point, px};

use super::structs::Mark;
use super::types::SLASH;

/// Maps a point on beui's 24-unit grid into `bounds`.
pub(crate) fn at(bounds: Bounds<Pixels>, x: f32, y: f32) -> Point<Pixels> {
    let unit = f32::from(bounds.size.width) / 24.;
    point(bounds.origin.x + px(x * unit), bounds.origin.y + px(y * unit))
}

/// The first `fraction` of the polyline `points`, by length.
pub(crate) fn partial(points: &[(f32, f32)], fraction: f32) -> Vec<(f32, f32)> {
    let seg = |a: (f32, f32), b: (f32, f32)| ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
    let total: f32 = points.windows(2).map(|w| seg(w[0], w[1])).sum();
    let mut left = total * fraction.clamp(0., 1.);
    let mut out = vec![points[0]];
    for w in points.windows(2) {
        let len = seg(w[0], w[1]);
        if left >= len {
            out.push(w[1]);
            left -= len;
        } else {
            let t = left / len;
            out.push((w[0].0 + (w[1].0 - w[0].0) * t, w[0].1 + (w[1].1 - w[0].1) * t));
            break;
        }
    }
    out
}

pub(crate) fn stroke(bounds: Bounds<Pixels>, points: &[(f32, f32)], width: f32, color: Hsla, window: &mut Window) {
    if points.len() < 2 {
        return;
    }
    let unit = f32::from(bounds.size.width) / 24.;
    let mut path = PathBuilder::stroke(px(width * unit));
    path.move_to(at(bounds, points[0].0, points[0].1));
    for p in &points[1..] {
        path.line_to(at(bounds, p.0, p.1));
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
}

/// Points around the radius-9 circle from `start` turns for `sweep` turns.
pub(crate) fn arc_points(start: f32, sweep: f32) -> Vec<(f32, f32)> {
    let steps = ((sweep.abs() * 64.).ceil() as usize).max(2);
    (0..=steps)
        .map(|i| {
            let a = (start + sweep * i as f32 / steps as f32) * TAU;
            (12. + 9. * a.cos(), 12. + 9. * a.sin())
        })
        .collect()
}

pub(super) fn paint(bounds: Bounds<Pixels>, m: Mark, window: &mut Window) {
    let unit = f32::from(bounds.size.width) / 24.;
    if m.fill_alpha > 0. {
        let mut path = PathBuilder::fill();
        let pts = arc_points(0., 1.);
        path.move_to(at(bounds, pts[0].0, pts[0].1));
        for p in &pts[1..] {
            path.line_to(at(bounds, p.0, p.1));
        }
        path.close();
        if let Ok(path) = path.build() {
            window.paint_path(path, m.color.opacity(m.color.a * m.fill_alpha));
        }
    }
    if m.ring_alpha > 0. {
        let mut path = PathBuilder::stroke(px(1.5 * unit));
        if m.dashed {
            path = path.dash_array(&[px(2. * unit), px(3. * unit)]);
        }
        let pts = arc_points(-0.25, 1.);
        path.move_to(at(bounds, pts[0].0, pts[0].1));
        for p in &pts[1..] {
            path.line_to(at(bounds, p.0, p.1));
        }
        if let Ok(path) = path.build() {
            window.paint_path(path, m.color.opacity(m.color.a * m.ring_alpha));
        }
    }
    if m.arc > 0.001 {
        stroke(bounds, &arc_points(m.arc_start - FRAC_PI_2 / TAU, m.arc), 2., m.color, window);
    }
    let glyph = m.glyph.unwrap_or(m.color);
    if m.check > 0.001 {
        stroke(bounds, &partial(&[(7.5, 12.25), (10.5, 15.25), (16.75, 8.75)], m.check), 2., glyph, window);
    }
    if m.cross > 0.001 {
        let half = (m.cross * 2.).min(1.);
        stroke(bounds, &partial(&[(8.5, 8.5), (15.5, 15.5)], half), 2., glyph, window);
        if m.cross > 0.5 {
            stroke(bounds, &partial(&[(15.5, 8.5), (8.5, 15.5)], m.cross * 2. - 1.), 2., glyph, window);
        }
    }
    if m.slash > 0.001 {
        stroke(bounds, &partial(&SLASH, m.slash), 2., glyph, window);
    }
}
