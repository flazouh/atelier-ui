use gpui_kit::{
    App,
    InteractiveElement,
    IntoElement,
    ParentElement,
    RenderOnce,
    Styled,
    Window,
    div,
    svg,
};

use crate::scale::px;
use crate::theme::ActiveTheme;
use super::types::{ASPECT, PATH, WIDTH};

#[derive(IntoElement)]
pub struct AtelierMark {
    pub(super) size: f32,
}

impl AtelierMark {
    pub fn new(size: f32) -> Self {
        Self { size }
    }
}

impl RenderOnce for AtelierMark {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let width = self.size * WIDTH;
        div()
            .debug_selector(|| "atelier-mark".into())
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(self.size))
            .rounded_full()
            .bg(theme.foreground)
            .child(svg().path(PATH).flex_none().w(px(width)).h(px(width / ASPECT)).text_color(theme.background))
    }
}
