use gpui_kit::{
    AnyView, App, AppContext, Context, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, base::Tooltip as BaseTooltip, div,
};

use crate::scale::px;
use crate::theme::ActiveTheme;
use super::helpers::surface;

pub struct Tooltip {
    pub(super) text: SharedString,
}

impl Tooltip {
    /// A builder for `.tooltip(...)`.
    pub fn text(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
        let text = text.into();
        move |_, cx| cx.new(|_| Tooltip { text: text.clone() }).into()
    }
}

impl Render for Tooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The margin keeps it off the pointer and the window edge.
        div().child(BaseTooltip::new("tooltip").m(px(6.)).child(surface(cx.theme()).debug_selector(|| "tooltip".into()).px(px(8.)).py(px(4.)).child(self.text.clone())))
    }
}
