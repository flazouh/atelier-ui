use gpui_kit::{
    App, Axis, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::button::{Button, ButtonSize, ButtonVariant};
use super::types::SEAM;
use super::helpers::segment_corners;

#[derive(IntoElement)]
pub struct ButtonGroup {
    pub(super) id: ElementId,
    pub(super) children: Vec<Button>,
    pub(super) variant: ButtonVariant,
    pub(super) size: ButtonSize,
    pub(super) layout: Axis,
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
