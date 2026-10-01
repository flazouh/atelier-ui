use std::panic::Location;

use gpui_kit::{
    AnyElement,
    App,
    Bounds,
    Element,
    ElementId,
    GlobalElementId,
    InspectorElementId,
    IntoElement,
    LayoutId,
    Pixels,
    Point,
    Window,
    point,
};

use crate::scale::px;
use crate::motion::{Channel, Curve, Spring};
use super::helpers::{moved, start_offset};

/// What the wrapper remembers between frames.
pub(super) struct Slot {
    /// Where the child's layout put it in the last frame, without the offset.
    pub(super) last: Option<Point<Pixels>>,
    pub(super) x: Channel,
    pub(super) y: Channel,
}

impl Default for Slot {
    fn default() -> Self {
        Self { last: None, x: Channel::new(0.), y: Channel::new(0.) }
    }
}

pub struct Shifted {
    pub(super) id: ElementId,
    pub(super) child: AnyElement,
    pub(super) spring: Spring,
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
