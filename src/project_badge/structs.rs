use std::path::PathBuf;

use gpui_kit::{
    App, IntoElement, ObjectFit, ParentElement, RenderOnce, SharedString, Styled, StyledImage,
    Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use super::types::SIZE;
use super::helpers::{fill, ink_on};

/// One colour of the palette: its name and its red, green and blue bytes.
pub struct Swatch {
    pub name: String,
    pub rgb: u32,
}

/// The badge: the project's icon when it has one, else its label on its colour.
#[derive(IntoElement)]
pub struct ProjectBadge {
    pub(super) label: SharedString,
    pub(super) color: usize,
    pub(super) icon: Option<PathBuf>,
}

impl ProjectBadge {
    pub fn new(label: impl Into<SharedString>, color: usize) -> Self {
        Self { label: label.into(), color, icon: None }
    }
    /// An image file that stands in for the letter.
    pub fn icon(mut self, icon: Option<PathBuf>) -> Self {
        self.icon = icon;
        self
    }
}

impl RenderOnce for ProjectBadge {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let fill = fill(self.color);
        let two = self.label.chars().count() > 1;
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(SIZE))
            .rounded(px(4.))
            .overflow_hidden()
            .when_some(self.icon.clone(), |d, icon| d.child(gpui_kit::img(icon).size(px(SIZE)).object_fit(ObjectFit::Contain)))
            .when(self.icon.is_none(), |d| {
                d.bg(fill)
                    .text_color(ink_on(fill))
                    .text_size(px(if two { 8. } else { 10. }))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .line_height(px(SIZE))
                    .child(self.label.clone())
            })
    }
}
