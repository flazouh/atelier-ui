//! beui's todo status marks (`TodoStatusIcon` in `components/agents/todo-list.tsx`), drawn as the same
//! SVG geometry on a 24-unit grid: a circle of radius 9, dashed `2 3` when pending, a progress arc when in
//! progress, and a check or cross stroke that draws itself in.

use std::f32::consts::{FRAC_PI_2, TAU};

use gpui_kit::{px, 
    App, Bounds, Hsla, IntoElement, PathBuilder, Pixels, Point, RenderOnce, Styled, Window, canvas, point, 
};

/// What the mark shows, with the amounts that animate between states.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub color: Hsla,
    /// The circle outline, and whether it is dashed (pending).
    pub ring_alpha: f32,
    pub dashed: bool,
    /// Fill wash inside the ring (completed uses 6%).
    pub fill_alpha: f32,
    /// How much of the progress arc is drawn, from 0 to 1, and where it starts in turns.
    pub arc: f32,
    pub arc_start: f32,
    /// How much of the check, the cross, and the slash is drawn, from 0 to 1.
    pub check: f32,
    pub cross: f32,
    pub slash: f32,
    /// The color of the check, cross, or slash. `None` draws them in `color`; `Some(page)` knocks them
    /// out of a filled disc in the page color.
    pub glyph: Option<Hsla>,
}

/// The ban glyph: one stroke across the disc.
pub(crate) const SLASH: [(f32, f32); 2] = [(8.5, 15.5), (15.5, 8.5)];

impl Mark {
    /// A quiet mark: nothing drawn.
    pub fn none(color: Hsla) -> Self {
        Self {
            color,
            ring_alpha: 0.,
            dashed: false,
            fill_alpha: 0.,
            arc: 0.,
            arc_start: 0.,
            check: 0.,
            cross: 0.,
            slash: 0.,
            glyph: None,
        }
    }

    /// A solid disc in `color`, with any glyph knocked out in `page`.
    pub fn filled(color: Hsla, page: Hsla) -> Self {
        Self { fill_alpha: 1., glyph: Some(page), ..Self::none(color) }
    }

    pub fn check(mut self, amount: f32) -> Self {
        self.check = amount;
        self
    }

    pub fn cross(mut self, amount: f32) -> Self {
        self.cross = amount;
        self
    }

    pub fn slash(mut self, amount: f32) -> Self {
        self.slash = amount;
        self
    }
}

#[derive(IntoElement)]
pub struct StatusMark {
    mark: Mark,
    size: Pixels,
}

impl StatusMark {
    pub fn new(mark: Mark, size: Pixels) -> Self {
        Self { mark, size }
    }
}

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

fn paint(bounds: Bounds<Pixels>, m: Mark, window: &mut Window) {
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

impl RenderOnce for StatusMark {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mark = self.mark;
        canvas(|_, _, _| {}, move |bounds, _, window, _| paint(bounds, mark, window)).size(self.size).flex_none()
    }
}

#[cfg(test)]
mod tests;
