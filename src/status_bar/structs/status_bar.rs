use gpui_kit::{ElementId, IntoElement};

use super::{ProviderGauge, SystemLoad, Work};

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
}
