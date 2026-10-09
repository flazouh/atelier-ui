use std::rc::Rc;

use gpui_kit::{App, ElementId, IntoElement, SharedString, Window};

use super::{ProviderGauge, SystemLoad, Work};

/// A press, as the app hears it.
pub type Press = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
/// The bar at the foot of the window.
pub struct StatusBar {
    pub(in super::super) id: ElementId,
    pub(in super::super) load: Option<SystemLoad>,
    pub(in super::super) work: Work,
    pub(in super::super) providers: Vec<ProviderGauge>,
    /// The widths of the cards that stand under the columns above the bar: the sidebar's and the right pane's.
    pub(in super::super) lead: Option<f32>,
    pub(in super::super) tail: Option<f32>,
    /// The app's version, and what a press on it does.
    pub(in super::super) version: Option<SharedString>,
    pub(in super::super) on_version: Option<Press>,
    /// What a press on the provider chips does.
    pub(in super::super) on_usage: Option<Press>,
}
