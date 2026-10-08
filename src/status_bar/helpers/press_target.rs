use gpui_kit::{
    AnyElement, Div, FocusHandle, InteractiveElement, IntoElement, Stateful,
    StatefulInteractiveElement, Styled, Window, prelude::FluentBuilder,
};

use crate::{focus::ring_shadow, theme::Theme};

use super::super::structs::Press;

/// Makes `target` a control of the bar: a hover tone, a tab stop with the focus ring when the keys reached it, and a
/// press on a click, Enter or Space. With no `press` it stays what it is.
pub fn press_target(
    target: Stateful<Div>,
    focus: &FocusHandle,
    press: Option<Press>,
    window: &Window,
    theme: &Theme,
) -> AnyElement {
    let Some(press) = press else {
        return target.into_any_element();
    };
    let keyed = focus.is_focused(window) && window.last_input_was_keyboard();
    let key = press.clone();
    target
        .cursor_pointer()
        .track_focus(&focus.clone().tab_stop(true))
        .hover(|s| {
            s.bg(theme.card_strong.opacity(0.6))
                .text_color(theme.foreground)
        })
        .when(keyed, |d| d.shadow(ring_shadow(theme, theme.card)))
        .on_click(move |_, window, cx| press(window, cx))
        .on_key_down(move |event, window, cx| {
            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                cx.stop_propagation();
                key(window, cx);
            }
        })
        .into_any_element()
}
