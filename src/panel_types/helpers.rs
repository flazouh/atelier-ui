use std::rc::Rc;

use gpui_kit::{AnyElement, App, AppContext, IntoElement, Window};

use super::structs::FnView;
use super::types::PanelContent;

/// Content drawn by `f`, as a view.
pub fn content_from(
    f: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    cx: &mut App,
) -> PanelContent {
    cx.new(|_| FnView(Rc::new(f))).into()
}

/// A panel's content. It is drawn afresh: a panel cached whole, with the composer's caret inside, drew a
/// frame every 50 to 130 ms on the HP. The app caches what inside it is costly (the conversation rows).
pub fn draw_content(content: &PanelContent) -> AnyElement {
    content.clone().into_any_element()
}

/// An element id from a word and the id of a panel, a project or a session.
pub(crate) fn element_id(prefix: &str, id: &str) -> gpui_kit::ElementId {
    gpui_kit::ElementId::Name(format!("{prefix}-{id}").into())
}
