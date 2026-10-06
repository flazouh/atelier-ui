/// Whether the Markdown body fades the text a stream adds to it. A plain tail draws itself, with its own fade, so the body
/// does not; and in the frame that tail ends and joins the body, the words were on the screen already, so they stay put.
/// Everything else the stream adds, such as a list, a code span, a link or bold words, fades as a plain tail does.
pub(super) fn fades_markdown(streaming: bool, fade_tail: bool, reduce_motion: bool, plain_tail: bool, was_plain_tail: bool) -> bool {
    streaming && fade_tail && !reduce_motion && !plain_tail && !was_plain_tail
}
#[cfg(test)]
mod tests {
    use super::fades_markdown;
    #[test]
    fn markdown_fades_while_it_streams_except_beside_a_plain_tail_and_as_one_joins_it() {
        assert!(fades_markdown(true, true, false, false, false), "a list or code span streaming in");
        assert!(!fades_markdown(true, true, false, true, false), "the plain tail draws its own fade");
        assert!(!fades_markdown(true, true, false, false, true), "the tail joined the body: nothing moves");
        assert!(!fades_markdown(false, true, false, false, false), "a finished answer shows at once");
        assert!(!fades_markdown(true, false, false, false, false), "the caller did not ask for it");
        assert!(!fades_markdown(true, true, true, false, false), "reduced motion");
    }
}
