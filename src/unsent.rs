//! Unsent Comments: comments the reader wrote on lines that are held, and shown to nobody else, until the
//! review carrying them is sent. Their count sits beside the control that sends them. The words never say
//! "pending", which readers take to mean somebody else owes a reply, nor "draft", which is already a pull
//! request's own state (GitQuiet's `CONTEXT.md`).

use std::rc::Rc;

use gpui_kit::{App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, };
use crate::scale::px;

use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::{Icon, IconName},
    theme::{ActiveTheme, radius},
    typography::TextSize,
};

/// Said under the count.
pub const WHO_SEES: &str = "Only you can see them until they are sent.";

pub fn count_text(count: usize) -> SharedString {
    if count == 1 { "1 unsent comment".into() } else { format!("{count} unsent comments").into() }
}

pub fn send_text(count: usize) -> SharedString {
    if count == 1 { "Send it".into() } else { format!("Send all {count}").into() }
}

/// The count of Unsent Comments and the control that sends them, on one row. Nothing shows with none.
#[derive(IntoElement)]
pub struct UnsentComments {
    id: ElementId,
    count: usize,
    on_send: Option<crate::review::ReviewHandler>,
}

impl UnsentComments {
    pub fn new(id: impl Into<ElementId>, count: usize) -> Self {
        Self { id: id.into(), count, on_send: None }
    }

    pub fn on_send(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_send = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for UnsentComments {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        if self.count == 0 {
            return div().into_any_element();
        }
        let send = Button::new(self.id.clone()).label(send_text(self.count)).variant(ButtonVariant::Primary).size(ButtonSize::Sm);
        let send = match self.on_send {
            Some(f) => send.on_click(move |_, window, cx| f(window, cx)),
            None => send.disabled(true),
        };
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(10.))
            .px(px(12.))
            .py(px(8.))
            .rounded(radius::lg())
            .bg(theme.card)
            .text_size(TextSize::Xs.font_size())
            .child(Icon::new(IconName::Edit).size(px(14.)).color(theme.warning))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(div().font_weight(gpui_kit::FontWeight::SEMIBOLD).text_color(theme.foreground.opacity(0.9)).child(count_text(self.count)))
                    .child(div().truncate().text_color(theme.muted_foreground).child(WHO_SEES)),
            )
            .child(send)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests;
