use gpui_kit::{Div, Styled, div};

use crate::{
    scale::px,
    theme::{Theme, radius},
};

/// A panel of the dashboard: the card tone, no border.
pub(super) fn panel(theme: &Theme) -> Div {
    div().flex().flex_col().min_w_0().px(px(20.)).py(px(18.)).rounded(radius::xxl()).bg(theme.card)
}
