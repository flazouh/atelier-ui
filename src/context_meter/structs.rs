use std::f32::consts::TAU;

use std::rc::Rc;

use gpui_kit::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement,
    Styled, Window, canvas, div, prelude::FluentBuilder, px,
};

use crate::{
    spinner::{RING_ALPHA, arc, dot, stroke},
    theme::ActiveTheme,
    tooltip::Tooltip,
};
use super::helpers::{fraction, ink, level, summary};
use super::types::{SIZE, SLOT, STEPS, STROKE};

#[derive(IntoElement)]
pub struct ContextMeter {
    id: ElementId,
    used: u64,
    window: u64,
    tip: bool,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
}

impl ContextMeter {
    /// `used` tokens of a window of `window`.
    pub fn new(id: impl Into<ElementId>, used: u64, window: u64) -> Self {
        Self { id: id.into(), used, window, tip: true, on_click: None }
    }

    /// Whether a hover tells the numbers. An owner that shows them in a panel turns it off while the panel is open.
    pub fn tip(mut self, tip: bool) -> Self {
        self.tip = tip;
        self
    }

    /// Makes the ring a button: a press runs `handler`.
    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
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
            .when(self.tip, |d| d.tooltip(Tooltip::text(summary(self.used, self.window))))
            .when_some(self.on_click, |d, click| d.cursor_pointer().on_click(move |event, window, cx| click(event, window, cx)))
            .child(ring)
    }
}
