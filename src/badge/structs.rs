use gpui_kit::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use crate::scale::px;
use crate::theme::ActiveTheme;
use super::types::Tone;

#[derive(IntoElement)]
pub struct Badge {
    pub(super) label: SharedString,
    pub(super) tone: Tone,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into(), tone: Tone::default() }
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (fill, text) = self.tone.colors(cx.theme());
        div()
            .flex()
            .flex_none()
            .items_center()
            .h(px(20.))
            .px(px(8.))
            .rounded_full()
            .bg(fill)
            .text_color(text)
            .text_size(px(11.))
            .line_height(px(18.))
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .child(self.label)
    }
}
