use std::f32::consts::TAU;

use gpui_kit::{Bounds, Hsla, PathBuilder, Pixels, Window};

use crate::scale::px;
use crate::{
    status_mark::{arc_points, at, partial, stroke},
    task_model::{Label, Priority, TaskStatus},
    theme::{Theme, mix},
};

/// The colour of a status's mark: quiet for work not begun, warm while it goes on, green in review, the
/// info tone when done.
pub fn status_color(status: TaskStatus, theme: &Theme) -> Hsla {
    match status {
        TaskStatus::Backlog | TaskStatus::Canceled => theme.muted_foreground,
        TaskStatus::Todo => mix(theme.muted_foreground, theme.foreground, 0.4),
        TaskStatus::InProgress => theme.warning,
        TaskStatus::InReview => theme.success,
        TaskStatus::Done => theme.info,
    }
}

/// The tone a label wears, from the eight the theme offers.
pub fn label_color(tone: u8, theme: &Theme) -> Hsla {
    match tone % 8 {
        0 => theme.info,
        1 => theme.success,
        2 => theme.warning,
        3 => theme.danger,
        4 => theme.accent,
        5 => theme.primary,
        6 => mix(theme.info, theme.danger, 0.5),
        _ => theme.muted_foreground,
    }
}

pub fn label_tone_color(label: &Label, theme: &Theme) -> Hsla {
    label_color(label.tone, theme)
}

/// A filled polygon through `points`, on the 24-unit grid.
fn fill_polygon(bounds: Bounds<Pixels>, points: &[(f32, f32)], color: Hsla, window: &mut Window) {
    if points.len() < 3 {
        return;
    }
    let mut path = PathBuilder::fill();
    path.move_to(at(bounds, points[0].0, points[0].1));
    for p in &points[1..] {
        path.line_to(at(bounds, p.0, p.1));
    }
    path.close();
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
}

pub(super) fn rect(x: f32, y: f32, w: f32, h: f32) -> [(f32, f32); 4] {
    [(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
}

/// The pie of radius `r` about the centre, from twelve o'clock, `fraction` of a turn.
pub(super) fn pie(fraction: f32, r: f32) -> Vec<(f32, f32)> {
    let steps = ((fraction * 48.).ceil() as usize).max(2);
    let mut points = vec![(12., 12.)];
    points.extend((0..=steps).map(|i| {
        let a = (-0.25 + fraction * i as f32 / steps as f32) * TAU;
        (12. + r * a.cos(), 12. + r * a.sin())
    }));
    points
}

pub(super) fn disc() -> Vec<(f32, f32)> {
    arc_points(0., 1.)
}

pub(super) fn ring(bounds: Bounds<Pixels>, color: Hsla, dashed: bool, window: &mut Window) {
    let unit = f32::from(bounds.size.width) / 24.;
    let mut path = PathBuilder::stroke(px(1.75 * unit));
    if dashed {
        path = path.dash_array(&[px(2. * unit), px(3. * unit)]);
    }
    let points = arc_points(-0.25, 1.);
    path.move_to(at(bounds, points[0].0, points[0].1));
    for p in &points[1..] {
        path.line_to(at(bounds, p.0, p.1));
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
}

pub(super) fn paint_status(bounds: Bounds<Pixels>, status: TaskStatus, theme: &Theme, window: &mut Window) {
    let color = status_color(status, theme);
    match status {
        TaskStatus::Backlog => ring(bounds, color, true, window),
        TaskStatus::Todo => ring(bounds, color, false, window),
        TaskStatus::InProgress | TaskStatus::InReview => {
            ring(bounds, color, false, window);
            fill_polygon(bounds, &pie(status.progress(), 5.5), color, window);
        }
        TaskStatus::Done => {
            fill_polygon(bounds, &disc(), color, window);
            stroke(bounds, &partial(&[(7.5, 12.25), (10.5, 15.25), (16.75, 8.75)], 1.), 2., theme.background, window);
        }
        TaskStatus::Canceled => {
            ring(bounds, color, false, window);
            stroke(bounds, &[(8.5, 8.5), (15.5, 15.5)], 1.75, color, window);
            stroke(bounds, &[(15.5, 8.5), (8.5, 15.5)], 1.75, color, window);
        }
    }
}

pub(super) fn paint_priority(
    bounds: Bounds<Pixels>,
    priority: Priority,
    theme: &Theme,
    window: &mut Window,
) {
    let faint = theme.faint();
    match priority {
        Priority::Urgent => {
            fill_polygon(bounds, &rect(3., 3., 18., 18.), theme.warning, window);
            fill_polygon(bounds, &rect(11., 7., 2., 6.5), theme.background, window);
            fill_polygon(bounds, &rect(11., 15., 2., 2.), theme.background, window);
        }
        Priority::None => {
            for x in [4., 10., 16.] {
                fill_polygon(bounds, &rect(x, 11., 4., 2.), faint, window);
            }
        }
        _ => {
            for (i, x) in [4., 10., 16.].into_iter().enumerate() {
                let height = 6. + 4. * i as f32;
                let on = i < priority.bars();
                let color = if on {
                    theme.foreground.opacity(0.85)
                } else {
                    faint
                };
                fill_polygon(bounds, &rect(x, 20. - height, 4., height), color, window);
            }
        }
    }
}
