use gpui_kit::{Context, IntoElement, ParentElement, Render, SharedString, Styled, Window, div};

use crate::scale::px;
use crate::{
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::TAB_HEIGHT;

/// What a tab being dragged shows beside the pointer: its title.
pub(crate) struct TabGhost(pub SharedString);

impl Render for TabGhost {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .px(px(10.))
            .h(px(TAB_HEIGHT))
            .flex()
            .items_center()
            .rounded(radius::md())
            .bg(theme.card_strong)
            .text_size(TextSize::Sm.font_size())
            .text_color(theme.foreground)
            .child(self.0.clone())
    }
}
