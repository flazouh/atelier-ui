//! A child that keeps its place on screen when the layout moves it, and glides to its new place on a spring:
//! Motion's `layout="position"`, done the way FLIP does it. After each frame the child's layout origin is
//! remembered. When the next layout puts it somewhere else, it is drawn, in that same frame, where it stood,
//! that is at the new place plus `old - new`, and the offset runs to zero on the spring. Under Reduce Motion
//! there is no offset: the child is at its new place at once.
//!
//! The offset is applied with the window's element offset during prepaint, so what the child holds, its
//! bounds, its hit boxes and what it paints, all move together. Only position is animated, not size.
//!
//! Do not put one under a scrolling container: a scroll moves the layout origin, and the child would follow
//! the scroll on a spring instead of with it.
use std::panic::Location;

use gpui_kit::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels, Point, Window,
    point, 
};
use crate::scale::px;

use crate::motion::{Channel, Curve, Spring};

/// A move smaller than this, in pixels, is not a move (rounding in layout).
const LEAST: f32 = 0.5;

/// What the wrapper remembers between frames.
struct Slot {
    /// Where the child's layout put it in the last frame, without the offset.
    last: Option<Point<Pixels>>,
    x: Channel,
    y: Channel,
}

impl Default for Slot {
    fn default() -> Self {
        Self { last: None, x: Channel::new(0.), y: Channel::new(0.) }
    }
}

pub struct Shifted {
    id: ElementId,
    child: AnyElement,
    spring: Spring,
}

/// `child`, gliding to wherever the layout moves it. `id` names it across frames, and must be the same each
/// frame for the same child (a chip's value, a toast's number).
pub fn shifted(id: impl Into<ElementId>, child: impl IntoElement) -> Shifted {
    Shifted { id: id.into(), child: child.into_any_element(), spring: Spring::LAYOUT }
}

impl Shifted {
    /// The spring the glide runs on; [`Spring::LAYOUT`] by default.
    pub fn spring(mut self, spring: Spring) -> Self {
        self.spring = spring;
        self
    }
}

impl IntoElement for Shifted {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

/// The offset that draws a child where it stood: from the layout origin it had, and the offset it wore, to the
/// origin it has now.
pub fn start_offset(was: Point<Pixels>, wore: Point<Pixels>, now: Point<Pixels>) -> Point<Pixels> {
    point(was.x + wore.x - now.x, was.y + wore.y - now.y)
}

/// Whether the child moved.
pub fn moved(was: Point<Pixels>, now: Point<Pixels>) -> bool {
    (f32::from(was.x) - f32::from(now.x)).abs() > LEAST || (f32::from(was.y) - f32::from(now.y)).abs() > LEAST
}

impl Element for Shifted {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, window: &mut Window, cx: &mut App) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(&mut self, id: Option<&GlobalElementId>, _: Option<&InspectorElementId>, bounds: Bounds<Pixels>, _: &mut (), window: &mut Window, cx: &mut App) {
        let reduce = cx.reduce_motion();
        let spring = self.spring;
        let offset = window.with_optional_element_state::<Slot, _>(id, |slot, window| {
            let mut slot = slot.flatten().unwrap_or_default();
            let now = bounds.origin;
            if reduce {
                slot.x = Channel::new(0.);
                slot.y = Channel::new(0.);
            } else if let Some(was) = slot.last.filter(|was| moved(*was, now)) {
                let wore = point(px(slot.x.value()), px(slot.y.value()));
                let start = start_offset(was, wore, now);
                slot.x = Channel::new(f32::from(start.x));
                slot.y = Channel::new(f32::from(start.y));
                slot.x.animate(0., Curve::Spring(spring), 0., false);
                slot.y.animate(0., Curve::Spring(spring), 0., false);
            }
            slot.last = Some(now);
            if slot.x.is_running() || slot.y.is_running() {
                window.request_animation_frame();
            }
            (point(px(slot.x.value()), px(slot.y.value())), Some(slot))
        });
        window.with_element_offset(offset, |window| self.child.prepaint(window, cx));
    }

    fn paint(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, _: Bounds<Pixels>, _: &mut (), _: &mut (), window: &mut Window, cx: &mut App) {
        self.child.paint(window, cx);
    }
}

#[cfg(test)]
mod tests;
