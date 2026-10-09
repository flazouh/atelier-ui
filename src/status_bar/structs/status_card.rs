use gpui_kit::AnyElement;

#[derive(gpui_kit::IntoElement)]
/// One card of the bar, as a panel is one: the card tone, the panels' corners and the bar's whole height. Its parts are
/// built by `load_parts`, `version_part`, `usage_part` and `work_part`, or are anything else that draws.
pub struct StatusCard {
    pub(in super::super) debug: &'static str,
    /// The width of the column above it; none takes what the others leave.
    pub(in super::super) width: Option<f32>,
    pub(in super::super) items: Vec<AnyElement>,
}
