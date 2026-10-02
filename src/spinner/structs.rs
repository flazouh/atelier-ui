use std::f32::consts::TAU;

use gpui_kit::{
    App, ElementId, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window,
    canvas, div, prelude::FluentBuilder, px,
};

use crate::{motion::now_millis, theme::ActiveTheme};
use super::types::{RING_ALPHA, STEPS, SWEEP};
use super::helpers::{arc, dot, pulse_at, stroke, stroke_width, turn_at};

#[derive(IntoElement)]
pub struct Spinner {
    id: ElementId,
    pub(super) size: Pixels,
    pub(super) color: Option<Hsla>,
}

impl Spinner {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), size: px(14.), color: None }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// Defaults to the foreground.
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl RenderOnce for Spinner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self.color.unwrap_or(cx.theme().foreground);
        let reduce = cx.reduce_motion();
        let size = f32::from(self.size);
        let millis = now_millis();
        let (start, opacity) = if reduce { (0., pulse_at(millis)) } else { (turn_at(millis), 1.) };
        // It draws again every frame while it is on screen.
        window.request_animation_frame();
        let _ = &self.id;
        div().flex_none().size(self.size).when(reduce, |d| d.opacity(opacity)).child(canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let width = stroke_width(size);
                let r = (size - width) / 2.;
                let c = (size / 2., size / 2.);
                stroke(&arc(c, r, 0., TAU, STEPS), width, bounds, color.opacity(RING_ALPHA), window);
                let arc_points = arc(c, r, start, SWEEP, STEPS / 4);
                stroke(&arc_points, width, bounds, color, window);
                for end in [arc_points.first(), arc_points.last()].into_iter().flatten() {
                    dot(*end, width / 2., bounds, color, window);
                }
            },
        ).size_full())
    }
}
