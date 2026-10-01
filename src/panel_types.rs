//! The data the agent panels take. The app owns what goes in a panel; atelier-ui lays panels out and draws their
//! tabs, their group headers and their fold.
use std::rc::Rc;

use gpui_kit::{AnyElement, AnyView, App, AppContext, Context, IntoElement, Render, SharedString, Window};

use crate::{
    agent_look::AgentLook,
    session_status::SessionStatus,
    sidebar_model::Location,
};

/// What a panel's project is called and where it lives, for a group header and a tab group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectLabel {
    pub id: SharedString,
    pub name: SharedString,
    pub location: Location,
}

/// A panel's content: a view the app made for it, so the panels draw it from its last frame until it
/// changes, and it keeps its state when it scrolls out of view.
pub type PanelContent = AnyView;

/// Draws an element.
type Draw = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// A view that draws `f`, for content with no view of its own (a story, a test).
struct FnView(Draw);

impl Render for FnView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        (self.0)(window, cx)
    }
}

/// Content drawn by `f`, as a view.
pub fn content_from(f: impl Fn(&mut Window, &mut App) -> AnyElement + 'static, cx: &mut App) -> PanelContent {
    cx.new(|_| FnView(Rc::new(f))).into()
}

/// A panel's content. It is drawn afresh: a panel cached whole, with the composer's caret inside, drew a
/// frame every 50 to 130 ms on the HP. The app caches what inside it is costly (the conversation rows).
pub fn draw_content(content: &PanelContent) -> AnyElement {
    content.clone().into_any_element()
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

/// How the open panels are shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    /// Columns in a row that scrolls sideways.
    #[default]
    SideBySide,
    /// One panel fills the area, and tabs pick it.
    Single,
}

/// What the app keeps between runs, per window: the layout, the grouping and each panel's width.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PanelsState {
    pub layout: Layout,
    pub grouped: bool,
    pub widths: Vec<(SharedString, f32)>,
}

/// What the panels tell the app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanelsEvent {
    /// A panel became the one the keys and the tabs act on.
    Activated(SharedString),
    /// The reader closed a panel. The app drops the session's panel from what it passes in.
    Closed(SharedString),
    /// The layout, the grouping or a width changed: the app may save [`PanelsState`].
    StateChanged,
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

/// An element id from a word and the id of a panel, a project or a session.
pub(crate) fn element_id(prefix: &str, id: &str) -> gpui_kit::ElementId {
    gpui_kit::ElementId::Name(format!("{prefix}-{id}").into())
}
