/// Whether the Markdown body fades the text a stream adds to it. A plain tail draws itself, with its own fade, so the body
/// does not; and in the frame that tail ends and joins the body, the words were on the screen already, so they stay put.
/// Everything else the stream adds, such as a list, a code span, a link or bold words, fades as a plain tail does.
pub(super) fn fades_markdown(streaming: bool, fade_tail: bool, reduce_motion: bool, plain_tail: bool, was_plain_tail: bool) -> bool {
    streaming && fade_tail && !reduce_motion && !plain_tail && !was_plain_tail
}
/// How a fenced code block looks in an answer: on the strong card tone, as the design system's own code block is, with rounded
/// corners. The Markdown view takes its code background from the theme's accent, which here is no tone of the card, so a block
/// would sit on the panel with nothing around it.
pub(super) fn code_block_style(theme: &crate::theme::Theme) -> gpui_kit::StyleRefinement {
    let corner = Some(gpui_kit::AbsoluteLength::from(crate::theme::radius::lg()));
    gpui_kit::StyleRefinement {
        background: Some(theme.card_strong.into()),
        corner_radii: gpui_kit::CornersRefinement { top_left: corner, top_right: corner, bottom_left: corner, bottom_right: corner },
        ..Default::default()
    }
}
#[cfg(test)]
mod tests {
    use super::{code_block_style, fades_markdown};
    #[test]
    fn markdown_fades_while_it_streams_except_beside_a_plain_tail_and_as_one_joins_it() {
        assert!(fades_markdown(true, true, false, false, false), "a list or code span streaming in");
        assert!(!fades_markdown(true, true, false, true, false), "the plain tail draws its own fade");
        assert!(!fades_markdown(true, true, false, false, true), "the tail joined the body: nothing moves");
        assert!(!fades_markdown(false, true, false, false, false), "a finished answer shows at once");
        assert!(!fades_markdown(true, false, false, false, false), "the caller did not ask for it");
        assert!(!fades_markdown(true, true, true, false, false), "reduced motion");
    }

    #[test]
    fn a_code_block_has_a_background_and_rounded_corners() {
        let theme = crate::theme::Theme::light();
        let style = code_block_style(&theme);
        assert!(style.background.is_some(), "a tone of its own");
        assert!(style.corner_radii.top_left.is_some() && style.corner_radii.bottom_right.is_some(), "rounded on every corner");
    }
}
