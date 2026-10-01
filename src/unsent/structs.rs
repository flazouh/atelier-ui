use std::rc::Rc;

use gpui_kit::{App, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window, div};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::{Icon, IconName},
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::WHO_SEES;
use super::helpers::{count_text, send_text};

/// The count of Unsent Comments and the control that sends them, on one row. Nothing shows with none.
#[derive(IntoElement)]
pub struct UnsentComments {
    pub(super) id: ElementId,
    pub(super) count: usize,
    pub(super) on_send: Option<crate::review::ReviewHandler>,
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
