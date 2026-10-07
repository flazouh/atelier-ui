use gpui_kit::{Context, IntoElement, Render, SharedString, Window};

use crate::{agent_look::AgentLook, session_status::SessionStatus, sidebar_model::Location};
use super::types::{Draw, Layout, PanelContent};

/// What a panel's project is called and where it lives, for a group header and a tab group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectLabel {
    pub id: SharedString,
    pub name: SharedString,
    pub location: Location,
    /// The project's badge, for a tab and a panel header; none draws the name.
    pub badge: Option<crate::sidebar_model::Badge>,
}

/// A view that draws `f`, for content with no view of its own (a story, a test).
pub(super) struct FnView(pub(super) Draw);

impl Render for FnView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        (self.0)(window, cx)
    }
}

#[derive(Clone)]
pub struct PanelData {
    pub id: SharedString,
    pub project: ProjectLabel,
    /// The session's title: what a tab says.
    pub title: SharedString,
    pub look: AgentLook,
    pub status: SessionStatus,
    pub content: PanelContent,
}

/// What the app keeps between runs, per window: the layout, the grouping and each panel's width.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PanelsState {
    pub layout: Layout,
    pub grouped: bool,
    pub widths: Vec<(SharedString, f32)>,
}

/// A tab being dragged.
#[derive(Clone)]
pub struct DraggedTab {
    pub id: SharedString,
    pub title: SharedString,
}

/// A column's edge being dragged.
#[derive(Clone)]
pub struct DraggedEdge {
    pub id: SharedString,
}
