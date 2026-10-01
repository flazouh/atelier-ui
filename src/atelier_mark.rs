//! The mark of atelier: a disc in the ink with an "A" cut out of it in the page's tone, so on the page it reads as
//! the primary button does, the page inverted. It has no colour of its own: a theme gives it both tones. The "A"
//! is `assets/atelier-mark.svg`, the shape tools/mac/make-icon.sh draws the app icon from.
use gpui_kit::{App, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window, div, svg};
use crate::scale::px;

use crate::theme::ActiveTheme;

/// Where [`crate::icon::Assets`] serves the "A".
pub const PATH: &str = "atelier/mark.svg";
/// The "A"'s width over its height, from its view box.
const ASPECT: f32 = 120. / 82.;
/// The "A"'s width over the disc's.
const WIDTH: f32 = 0.6;

pub(crate) fn bytes(path: &str) -> Option<&'static [u8]> {
    (path == PATH).then_some(include_bytes!("../assets/atelier-mark.svg"))
}

#[derive(IntoElement)]
pub struct AtelierMark {
    size: f32,
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
