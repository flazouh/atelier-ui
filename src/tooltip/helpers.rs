use gpui_kit::{Div, Styled, div};

use crate::scale::px;
use crate::theme::{Theme, radius};

/// The fill, text, and corners of a tooltip.
pub(super) fn surface(theme: &Theme) -> Div {
    div()
        .rounded(radius::md())
        .bg(theme.foreground)
        .text_color(theme.background)
        .shadow_md()
        .text_size(px(11.))
        .line_height(px(16.))
}
