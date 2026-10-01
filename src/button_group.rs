//! ButtonGroup: buttons joined into one control, as gpui-component's `ButtonGroup`
//! (`vendor/gpui-component/src/button/button_group.rs`) joins them, reskinned for atelier-ui. The segments
//! touch, the group keeps the button's corner on its outer corners only, and every segment takes the
//! group's variant and size, so they share one height, one fill and threadmail's press, each on the
//! segment pressed.
//!
//! Borderless: no line parts the segments. A 1px seam does, the surface under the group showing
//! through, so it reads as one button with parts on the page and on a card, in every theme.
//!
//! Each segment keeps its own state: one may be disabled while the others stay live, as the merge
//! button's action greys while its arrow still opens the menu. Each segment is a stop in the Tab order.

use gpui_kit::{App, Axis, Corners, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window, div, prelude::FluentBuilder, };
use crate::scale::px;

use crate::button::{Button, ButtonSize, ButtonVariant};

/// The seam between two segments.
pub const SEAM: f32 = 1.;

#[derive(IntoElement)]
pub struct ButtonGroup {
    id: ElementId,
    children: Vec<Button>,
    variant: ButtonVariant,
    size: ButtonSize,
    layout: Axis,
}

impl ButtonGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), children: Vec::new(), variant: ButtonVariant::default(), size: ButtonSize::default(), layout: Axis::Horizontal }
    }

    pub fn child(mut self, child: Button) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Button>) -> Self {
        self.children.extend(children);
        self
    }

    /// The look every segment takes.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// The size every segment takes.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Side by side (the default) or stacked.
    pub fn layout(mut self, layout: Axis) -> Self {
        self.layout = layout;
        self
    }
}

/// The corners segment `index` of `len` rounds: the group's outer corners only, as gpui-component's
/// group sets them.
pub fn segment_corners(index: usize, len: usize, layout: Axis) -> Corners<bool> {
    let vertical = layout == Axis::Vertical;
    let (first, last) = (index == 0, index + 1 == len);
    Corners {
        top_left: first,
        top_right: if vertical { first } else { last },
        bottom_left: if vertical { last } else { first },
        bottom_right: last,
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let len = self.children.len();
        let vertical = self.layout == Axis::Vertical;
        let (variant, size, layout) = (self.variant, self.size, self.layout);
        div()
            .id(self.id)
            .flex()
            .flex_none()
            .when(vertical, |d| d.flex_col())
            .when(!vertical, |d| d.items_center())
            .gap(px(SEAM))
            .children(self.children.into_iter().enumerate().map(move |(i, child)| {
                child.variant(variant).size(size).corners(segment_corners(i, len, layout)).focusable(true)
            }))
    }
}

#[cfg(test)]
mod tests;
