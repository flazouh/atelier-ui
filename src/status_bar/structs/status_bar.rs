use gpui_kit::{ElementId, IntoElement};

use super::{ProviderGauge, SystemLoad, Work};

#[derive(IntoElement)]
/// The bar at the foot of the window.
pub struct StatusBar {
    pub(in super::super) id: ElementId,
    pub(in super::super) load: Option<SystemLoad>,
    pub(in super::super) work: Work,
    pub(in super::super) providers: Vec<ProviderGauge>,
}
