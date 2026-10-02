//! Amber waves for a voice: while it records, a few translucent ribbons swell and ripple with the level of the
//! microphone, and settle to one thin line when it is quiet or off.
//!
//! SPIKE: the first look at a recording state for the prompt input. The level comes from the caller, 0 to 1.
//!
//! - Four ribbons, each a mirrored sine pair that drifts at its own speed and direction, filled in amber at low
//!   opacity so that where they overlap the amber deepens. A thin bright line rides the first ribbon's edge.
//! - A window `sin(pi x)^1.6` tapers every ribbon to nothing at both ends, so the waves float in their space.
//! - The level is smoothed with a fast attack and a slow release, so a word lands at once and fades gently.
//! - The phase moves faster the louder the voice, which makes the waves feel pushed by it.
//! - Reduce Motion: the ribbons hold still at their current size.
use std::{f32::consts::PI, time::Instant};

use gpui_kit::{App, ElementId, Hsla, IntoElement, ParentElement, PathBuilder, Pixels, RenderOnce, Styled, Window, canvas, div, hsla, point, px};

use crate::scale::px as scaled;

/// Amber, `hsl(38 100% 55%)`.
pub fn amber() -> Hsla {
    hsla(38. / 360., 1., 0.55, 1.)
}

/// One ribbon: how many waves fit across, how fast and which way they drift, where they start, how opaque, how tall.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ribbon {
    pub waves: f32,
    pub speed: f32,
    pub phase: f32,
    pub alpha: f32,
    pub gain: f32,
}

pub const RIBBONS: [Ribbon; 4] = [
    Ribbon { waves: 1.6, speed: 1.0, phase: 0.0, alpha: 0.22, gain: 1.00 },
    Ribbon { waves: 2.4, speed: -1.3, phase: 1.7, alpha: 0.17, gain: 0.80 },
    Ribbon { waves: 3.3, speed: 1.9, phase: 3.1, alpha: 0.13, gain: 0.62 },
    Ribbon { waves: 4.6, speed: -2.4, phase: 4.4, alpha: 0.10, gain: 0.45 },
];

/// The quietest the ribbons get while it is on: a thin line, so the field never looks dead.
pub const FLOOR: f32 = 0.06;

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

/// The level after `dt` seconds toward `target`: it rises quickly and falls slowly.
pub fn smooth(current: f32, target: f32, dt: f32) -> f32 {
    let rate = if target > current { 18. } else { 4.5 };
    current + (target - current) * (1. - (-rate * dt).exp())
}

struct WaveState {
    level: f32,
    phase: f32,
    at: Instant,
}

#[derive(IntoElement)]
pub struct VoiceWaves {
    id: ElementId,
    level: f32,
    height: Pixels,
    color: Hsla,
}

impl VoiceWaves {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), level: 0., height: px(40.), color: amber() }
    }

    /// The microphone's level now, 0 to 1.
    pub fn level(mut self, level: f32) -> Self {
        self.level = level;
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = color;
        self
    }
}

impl RenderOnce for VoiceWaves {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let state = window.use_keyed_state(self.id, cx, |_, _| WaveState { level: 0., phase: 0., at: Instant::now() });
        let (level, phase) = state.update(cx, |s, _| {
            let now = Instant::now();
            let dt = now.duration_since(s.at).as_secs_f32().min(0.1);
            s.at = now;
            s.level = if reduce { self.level } else { smooth(s.level, self.level, dt) };
            if !reduce {
                s.phase += dt * (1.6 + 5. * s.level);
            }
            (s.level, s.phase)
        });
        if !reduce {
            window.request_animation_frame();
        }
        let color = self.color;
        div().w_full().h(self.height).child(
            canvas(
                |_, _, _| {},
                move |bounds, _, window, _| {
                    let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
                    if w < 4. || h < 2. {
                        return;
                    }
                    let steps = ((w / 3.) as usize).clamp(24, 160);
                    let mid = h / 2.;
                    let at = |x: f32, y: f32| point(bounds.origin.x + scaled(x * w), bounds.origin.y + scaled(y));
                    for (i, ribbon) in RIBBONS.iter().enumerate() {
                        let amps: Vec<f32> = (0..=steps).map(|k| amplitude(*ribbon, k as f32 / steps as f32, level, phase)).collect();
                        let mut fill = PathBuilder::fill();
                        fill.move_to(at(0., mid));
                        for (k, a) in amps.iter().enumerate() {
                            fill.line_to(at(k as f32 / steps as f32, mid - a * mid));
                        }
                        for (k, a) in amps.iter().enumerate().rev() {
                            fill.line_to(at(k as f32 / steps as f32, mid + a * mid));
                        }
                        fill.close();
                        if let Ok(path) = fill.build() {
                            window.paint_path(path, color.opacity(ribbon.alpha));
                        }
                        // The first ribbon's upper and lower edge catch the light.
                        if i == 0 {
                            for side in [-1., 1.] {
                                let mut edge = PathBuilder::stroke(scaled(1.25));
                                for (k, a) in amps.iter().enumerate() {
                                    let p = at(k as f32 / steps as f32, mid + side * a * mid);
                                    if k == 0 { edge.move_to(p) } else { edge.line_to(p) }
                                }
                                if let Ok(path) = edge.build() {
                                    window.paint_path(path, color.opacity(0.85));
                                }
                            }
                        }
                    }
                },
            )
            .size_full(),
        )
    }
}

#[cfg(test)]
mod tests;
