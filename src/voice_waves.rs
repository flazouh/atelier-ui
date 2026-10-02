//! Amber bars for a voice: while it records, a row of rounded bars swells and ripples with the level of the
//! microphone, and settles to a line of small dots when it is quiet or off.
//!
//! SPIKE: the first look at a recording state for the prompt input. The level comes from the caller, 0 to 1.
//!
//! - The shape of the field comes from four drifting sine ribbons, each at its own speed and direction. Each bar
//!   reads the ribbons at its own spot, so the crests travel through the row instead of every bar bouncing alone.
//! - A window `sin(pi x)^1.6` tapers the row to dots at both ends, so the bars float in their space.
//! - The level is smoothed with a fast attack and a slow release, so a word lands at once and fades gently.
//! - The phase moves faster the louder the voice, which makes the waves feel pushed by it.
//! - Taller bars are also brighter, which gives the row depth without a second color.
//! - Reduce Motion: the bars hold still at their current size.
use std::{f32::consts::PI, time::Instant};

use gpui_kit::{App, ElementId, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window, div, hsla, px};

use crate::{scale::px as scaled, theme::ActiveTheme};

/// A bar never gets wider than this, so a wide row keeps its look and centers instead of growing fat bars.
pub const MAX_BAR_WIDTH: f32 = 6.;

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

/// How tall the bar at `x` (0 to 1 across the row) stands, as a fraction of the row's height: the four ribbons
/// read at one spot and averaged, then lifted so a loud voice fills the row.
pub fn bar(x: f32, level: f32, phase: f32) -> f32 {
    let (sum, gains) = RIBBONS.iter().fold((0., 0.), |(sum, gains), r| (sum + amplitude(*r, x, level, phase), gains + r.gain));
    (sum / gains * BAR_LIFT).clamp(0., 1.)
}

/// What the ribbons' average is multiplied by, because the average of four ribbons that rarely peak together sits
/// well under the tallest one.
pub const BAR_LIFT: f32 = 1.9;

/// The shortest a bar gets, in pixels: a dot as wide as the bar, so a quiet row still reads as a row of bars.
pub const MIN_BAR: f32 = 3.;

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
    color: Option<Hsla>,
    bars: usize,
    gap: f32,
}

impl VoiceWaves {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), level: 0., height: px(40.), color: None, bars: 36, gap: 3. }
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

    /// The bars' color. Without one, [`amber_for`] the theme.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// How many bars stand in the row; they share the width evenly, up to [`MAX_BAR_WIDTH`] each.
    pub fn bars(mut self, bars: usize) -> Self {
        self.bars = bars.max(3);
        self
    }

    /// The space between bars, in pixels.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
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
        let color = self.color.unwrap_or_else(|| amber_for(cx.theme()));
        let (height, count) = (f32::from(self.height), self.bars);
        let bars = (0..count).map(|k| {
            let a = bar((k as f32 + 0.5) / count as f32, level, phase);
            let h = (a * height).max(MIN_BAR.min(height));
            div()
                .flex_1()
                .max_w(scaled(MAX_BAR_WIDTH))
                .h(scaled(h))
                .rounded(scaled(MAX_BAR_WIDTH / 2.))
                .bg(color.opacity(0.4 + 0.6 * a))
        });
        div().w_full().h(self.height).flex().flex_row().items_center().justify_center().gap(scaled(self.gap)).children(bars)
    }
}

#[cfg(test)]
mod tests;
