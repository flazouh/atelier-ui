/// An element id from a word and the id of a project or a session.
pub(super) fn name(prefix: &str, id: &str) -> gpui_kit::ElementId {
    gpui_kit::ElementId::Name(format!("{prefix}-{id}").into())
}
