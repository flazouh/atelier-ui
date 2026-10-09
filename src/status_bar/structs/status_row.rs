use gpui_kit::{AnyElement, ElementId};

#[derive(gpui_kit::IntoElement)]
/// The bar at the foot of the window, as a row of cards a gap apart. [`StatusBar`](super::StatusBar) lays three out
/// itself; an app that has its own cards puts them in this.
pub struct StatusRow {
    pub(in super::super) id: ElementId,
    pub(in super::super) cards: Vec<AnyElement>,
}
