use gpui_kit::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
};

use crate::{panel_layout::GAP, scale::px, theme::ActiveTheme, typography::TextSize};

use super::super::{consts::HEIGHT, structs::StatusRow};

impl StatusRow {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            cards: Vec::new(),
        }
    }

    /// The cards, left to right. A [`StatusCard`](super::super::StatusCard) is one.
    pub fn children<C: IntoElement>(mut self, cards: impl IntoIterator<Item = C>) -> Self {
        self.cards
            .extend(cards.into_iter().map(IntoElement::into_any_element));
        self
    }
}

impl RenderOnce for StatusRow {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .id(self.id)
            .debug_selector(|| "status-bar".into())
            .flex()
            .flex_none()
            .items_center()
            .gap(px(GAP))
            .h(px(HEIGHT))
            .text_size(TextSize::Xs.font_size())
            .text_color(theme.muted_foreground)
            .children(self.cards)
    }
}
