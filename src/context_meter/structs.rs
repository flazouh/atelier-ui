use std::f32::consts::TAU;

use gpui_kit::{
    App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement,
    Styled, Window, canvas, div, px,
};

use crate::{
    spinner::{RING_ALPHA, arc, dot, stroke},
    theme::{ActiveTheme, Theme},
    tooltip::Tooltip,
};
use super::helpers::{fraction, level, summary};
use super::types::{Level, SIZE, SLOT, STEPS, STROKE};

#[derive(IntoElement)]
pub struct ContextMeter {
    id: ElementId,
    used: u64,
    window: u64,
}

impl ContextMeter {
    /// `used` tokens of a window of `window`.
    pub fn new(id: impl Into<ElementId>, used: u64, window: u64) -> Self {
        Self { id: id.into(), used, window }
    }
}

fn ink(level: Level, theme: &Theme) -> Hsla {
    match level {
        Level::Room => theme.muted_foreground,
        Level::Filling => theme.warning,
        Level::Full => theme.danger,
    }
}

impl RenderOnce for ContextMeter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let filled = fraction(self.used, self.window);
        let color = ink(level(filled), cx.theme());
        let ring = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let radius = (SIZE - STROKE) / 2.;
                let centre = (SIZE / 2., SIZE / 2.);
                stroke(&arc(centre, radius, 0., TAU, STEPS), STROKE, bounds, color.opacity(RING_ALPHA), window);
                if filled <= 0. {
                    return;
                }
                let steps = ((STEPS as f32 * filled).ceil() as usize).max(1);
                let points = arc(centre, radius, 0., TAU * filled, steps);
                stroke(&points, STROKE, bounds, color, window);
                for end in [points.first(), points.last()].into_iter().flatten() {
                    dot(*end, STROKE / 2., bounds, color, window);
                }
            },
        )
        .size(px(SIZE));
        div()
            .id(self.id)
            .debug_selector(|| "context-meter".into())
            .flex_none()
            .size(px(SLOT))
            .flex()
            .items_center()
            .justify_center()
            .tooltip(Tooltip::text(summary(self.used, self.window)))
            .child(ring)
    }
}
