use gpui_kit::{IntoElement, ParentElement, Styled, div, prelude::FluentBuilder};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize},
    review::{ReviewHandler},
    theme::{Theme},
    typography::{MONO_FONT_FAMILY, TextSize},
};

pub(super) fn counts(id: impl Into<gpui_kit::ElementId>, added: usize, removed: usize, theme: &Theme) -> impl IntoElement {
    let id = id.into();
    let count = |text: String, added: bool| {
        div().font_family(MONO_FONT_FAMILY).text_size(TextSize::Xs.font_size()).text_color(theme.diff_color(added)).child(crate::Digits::new((id.clone(), if added { "added" } else { "removed" }), text, TextSize::Xs.font_size()))
    };
    div()
        .relative()
        .flex()
        .flex_none()
        .gap(px(6.))
        .when(added > 0, |d| d.child(count(format!("+{added}"), true)))
        .when(removed > 0, |d| d.child(count(format!("\u{2212}{removed}"), false)))
}

pub(super) fn button(button: Button, handler: &Option<ReviewHandler>) -> Button {
    let button = button.size(ButtonSize::Sm);
    match handler.clone() {
        Some(f) => button.on_click(move |_, window, cx| f(window, cx)),
        None => button.disabled(true),
    }
}
