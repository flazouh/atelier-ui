use gpui_kit::ElementId;

/// The id of a part of the dashboard, named under the dashboard's own.
pub(super) fn key(id: &ElementId, name: String) -> ElementId {
    ElementId::from((id.clone(), name))
}
