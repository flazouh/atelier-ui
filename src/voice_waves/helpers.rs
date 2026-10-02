use std::f32::consts::PI;

use gpui_kit::{Hsla, hsla};

use super::structs::Ribbon;
use super::types::{BAR_LIFT, FLOOR, RIBBONS};

/// Amber, `hsl(38 100% 55%)`.
pub fn amber() -> Hsla {
    hsla(38. / 360., 1., 0.55, 1.)
}

/// Amber for this theme: the bright one on a dark page, a deeper one on a light page, where the bright one is too pale to read
/// (1.9 to 1 against white, against 4.0 to 1 for the deeper one).
pub fn amber_for(theme: &crate::theme::Theme) -> Hsla {
    match theme.appearance {
        crate::theme::Appearance::Dark => amber(),
        crate::theme::Appearance::Light => hsla(33. / 360., 0.95, 0.38, 1.),
    }
}

/// What a mark on [`amber_for`] is drawn in: near-black on the bright amber, white on the deep one.
pub fn on_amber(theme: &crate::theme::Theme) -> Hsla {
    match theme.appearance {
        crate::theme::Appearance::Dark => hsla(38. / 360., 0.7, 0.1, 1.),
        crate::theme::Appearance::Light => hsla(0., 0., 1., 1.),
    }
}

/// Zero at both ends, one in the middle.
pub fn window(x: f32) -> f32 {
    (PI * x).sin().max(0.).powf(1.6)
}

/// Half the ribbon's thickness at `x` (0 to 1 across), as a fraction of the half-height, for a smoothed `level`
/// and a `phase` in radians.
pub fn amplitude(ribbon: Ribbon, x: f32, level: f32, phase: f32) -> f32 {
    let level = level.clamp(0., 1.).max(FLOOR);
    let angle = ribbon.waves * 2. * PI * x + phase * ribbon.speed + ribbon.phase;
    // A second, slower sine bends the crest so no two peaks match.
    let body = 0.62 + 0.38 * angle.sin() * (0.5 * angle + 1.3).cos();
    (window(x) * level * ribbon.gain * body.abs()).clamp(0., 1.)
}

/// How tall the bar at `x` (0 to 1 across the row) stands, as a fraction of the row's height: the four ribbons
/// read at one spot and averaged, then lifted so a loud voice fills the row.
pub fn bar(x: f32, level: f32, phase: f32) -> f32 {
    let (sum, gains) = RIBBONS.iter().fold((0., 0.), |(sum, gains), r| (sum + amplitude(*r, x, level, phase), gains + r.gain));
    (sum / gains * BAR_LIFT).clamp(0., 1.)
}

/// The level after `dt` seconds toward `target`: it rises quickly and falls slowly.
pub fn smooth(current: f32, target: f32, dt: f32) -> f32 {
    let rate = if target > current { 18. } else { 4.5 };
    current + (target - current) * (1. - (-rate * dt).exp())
}
