use std::ops::Range;

use gpui_kit::{AnyElement, IntoElement, ParentElement, Styled, div};

use crate::scale::px;
use crate::{theme::Theme, typography::TextSize};

/// The rows of `total` that a body clipped to `rows` shows: the newest while it streams, as a terminal does, and
/// the first once it is done.
pub(crate) fn window(total: usize, rows: usize, newest: bool) -> Range<usize> {
    if total <= rows {
        0..total
    } else if newest {
        total - rows..total
    } else {
        0..rows
    }
}

/// The line under a clipped body that says what a press does.
pub(crate) fn hint(hidden: usize, expanded: bool, theme: &Theme) -> AnyElement {
    let words = match (expanded, hidden) {
        (true, _) => "Show less".to_string(),
        (false, 1) => "1 more line".to_string(),
        (false, n) => format!("{n} more lines"),
    };
    div()
        .flex()
        .items_center()
        .h(px(24.))
        .px(px(12.))
        .text_size(TextSize::Xs.font_size())
        .text_color(theme.faint())
        .child(words)
        .into_any_element()
}
