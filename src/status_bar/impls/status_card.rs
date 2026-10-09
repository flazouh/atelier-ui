use gpui_kit::{
    AnyElement, App, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window,
    div, prelude::FluentBuilder,
};

use crate::{
    scale::px,
    theme::{ActiveTheme, radius},
};

use super::super::structs::StatusCard;

impl StatusCard {
    /// A card whose test selector is `debug`.
    pub fn new(debug: &'static str) -> Self {
        Self {
            debug,
            width: None,
            items: Vec::new(),
        }
    }

    /// The width of the column above the card. None takes what the other cards leave.
    pub fn width(mut self, width: Option<f32>) -> Self {
        self.width = width;
        self
    }

    pub fn children(mut self, items: impl IntoIterator<Item = AnyElement>) -> Self {
        self.items.extend(items);
        self
    }
}

impl RenderOnce for StatusCard {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let (debug, width) = (self.debug, self.width);
        div()
            .debug_selector(move || debug.into())
            .flex()
            .items_center()
            .gap(px(10.))
            .h_full()
            .px(px(10.))
            .rounded(radius::lg())
            .bg(cx.theme().card)
            .overflow_hidden()
            .when_some(width, |d, w| d.flex_none().w(px(w)))
            .when(width.is_none(), |d| d.flex_1().min_w_0())
            .children(self.items)
    }
}
