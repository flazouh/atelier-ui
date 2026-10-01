use super::{CancelComment, SubmitComment};

use gpui_kit::{App, KeyBinding, Styled, div};

use crate::scale::px;
use crate::{
    theme::{ActiveTheme, radius},
    typography::{FONT_FAMILY},
};

/// Bound after the inline review's keys: inside the composer, `secondary-enter` sends the comment, not
/// the review's accept, since GPUI lets the later binding win at the same depth.
pub(crate) fn bind_keys(cx: &mut App) {
    let context = Some("LineComposer > Input");
    cx.bind_keys([
        KeyBinding::new("secondary-enter", SubmitComment, context),
        KeyBinding::new("escape", CancelComment, context),
    ]);
}

/// Space around a thread or a composer, inside its gap. The gap sits in the editor, whose text is
/// mono; a comment is prose, so it sets the sans font back.
pub(super) fn gap_frame() -> gpui_kit::Div {
    div().pl(px(12.)).pr(px(24.)).py(px(8.)).font_family(FONT_FAMILY)
}

/// A thread's card: one step above the review's own card, which the editor sits on.
pub(super) fn card(cx: &App) -> gpui_kit::Div {
    let theme = cx.theme();
    div().flex().flex_col().gap(px(10.)).p(px(12.)).rounded(radius::xl()).bg(theme.card_strong).max_w(px(640.))
}
