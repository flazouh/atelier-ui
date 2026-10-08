use gpui_kit::{
    Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window, div,
};

use super::types::TAB_HEIGHT;
use crate::agent_panels::AgentPanels;
use crate::scale::px;
use crate::{
    theme::{ActiveTheme, radius},
    typography::TextSize,
};

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

/// The single view's tab bar as a view of its own, for an owner that draws it elsewhere than above the panel (in its
/// title bar). It follows the panels, and it is the panels that hold the tabs, their order and their scroll.
pub struct TabStrip {
    panels: Entity<AgentPanels>,
}

impl TabStrip {
    pub fn new(panels: Entity<AgentPanels>, cx: &mut Context<Self>) -> Self {
        cx.observe(&panels, |_, _, cx| cx.notify()).detach();
        Self { panels }
    }
}

impl Render for TabStrip {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.panels
            .update(cx, |panels, cx| panels.tab_strip(window, cx))
    }
}
