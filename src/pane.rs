//! Pane headers. The app draws its own title bar, so every header also moves the window.

use gpui_kit::{
    App, Div, FontWeight, InteractiveElement, MouseButton, ParentElement, Pixels, SharedString,
    Styled, div, px,
};

use crate::{theme::ActiveTheme, typography::TextSize};

/// Header height, tall enough to clear the macOS window buttons.
/// The height of a pane's head, at the interface's scale.
pub fn pane_header_height() -> Pixels {
    crate::scale::px(44.)
}

/// A pane header with a title. Add buttons after it; put `drag_space()` in the empty part. It has no rule
/// under it: the content starts after its height.
pub fn pane_header(title: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(4.))
        .h(pane_header_height())
        .pl(px(16.))
        .pr(px(8.))
        .text_color(cx.theme().foreground)
        .child(
            div()
                .text_size(TextSize::Sm.font_size())
                .font_weight(FontWeight::MEDIUM)
                .child(title.into()),
        )
}

/// Empty space that moves the window when dragged. Keep buttons outside it, so clicks still reach them.
pub fn drag_space() -> Div {
    div()
        .flex_1()
        .h_full()
        .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move())
}
