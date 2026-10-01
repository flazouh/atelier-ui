use std::rc::Rc;

use gpui_kit::{AnyElement, AnyView, App, SharedString, Window};

/// A panel's content: a view the app made for it, so the panels draw it from its last frame until it
/// changes, and it keeps its state when it scrolls out of view.
pub type PanelContent = AnyView;

/// Draws an element.
pub(super) type Draw = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// How the open panels are shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    /// Columns in a row that scrolls sideways.
    #[default]
    SideBySide,
    /// One panel fills the area, and tabs pick it.
    Single,
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
