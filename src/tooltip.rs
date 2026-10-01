//! A short label on hover: gpui-base's unstyled `Tooltip` popup, in
//! the page inverted (dark ink in light, cream in dark), as the primary button is: no border, and one
//! step of shadow so it lifts off a card.
//!
//! Hand [`Tooltip::text`] to GPUI's own `.tooltip(...)` on a stateful element; GPUI owns the delay and
//! the placement.

use gpui_kit::{
    AnyView, App, AppContext, Context, Div, InteractiveElement, IntoElement, ParentElement, Render, SharedString, Styled, Window,
    base::Tooltip as BaseTooltip, div, 
};
use crate::scale::px;

use crate::theme::{ActiveTheme, Theme, radius};

/// The fill, text, and corners of a tooltip.
fn surface(theme: &Theme) -> Div {
    div()
        .rounded(radius::md())
        .bg(theme.foreground)
        .text_color(theme.background)
        .shadow_md()
        .text_size(px(11.))
        .line_height(px(16.))
}

pub struct Tooltip {
    text: SharedString,
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
