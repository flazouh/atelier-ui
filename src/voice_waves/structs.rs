use std::time::Instant;

use gpui_kit::{
    App, ElementId, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window, div, px,
};

use super::helpers::{amber_for, bar, smooth};
use super::types::{MAX_BAR_WIDTH, MIN_BAR};
use crate::{scale::px as scaled, theme::ActiveTheme};

/// One ribbon: how many waves fit across, how fast and which way they drift, where they start, how opaque, how tall.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ribbon {
    pub waves: f32,
    pub speed: f32,
    pub phase: f32,
    pub alpha: f32,
    pub gain: f32,
}

pub(super) struct WaveState {
    pub(super) level: f32,
    pub(super) phase: f32,
    pub(super) at: Instant,
}

#[derive(IntoElement)]
pub struct VoiceWaves {
    pub(super) id: ElementId,
    pub(super) level: f32,
    pub(super) height: Pixels,
    pub(super) color: Option<Hsla>,
    pub(super) bars: usize,
    pub(super) gap: f32,
}

impl VoiceWaves {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            level: 0.,
            height: px(40.),
            color: None,
            bars: 36,
            gap: 3.,
        }
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
        let state = window.use_keyed_state(self.id, cx, |_, _| WaveState {
            level: 0.,
            phase: 0.,
            at: Instant::now(),
        });
        let (level, phase) = state.update(cx, |s, _| {
            let now = Instant::now();
            let dt = now.duration_since(s.at).as_secs_f32().min(0.1);
            s.at = now;
            s.level = if reduce {
                self.level
            } else {
                smooth(s.level, self.level, dt)
            };
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
        div()
            .w_full()
            .h(self.height)
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(scaled(self.gap))
            .children(bars)
    }
}
