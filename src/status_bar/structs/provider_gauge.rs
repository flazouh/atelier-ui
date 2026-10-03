use gpui_kit::SharedString;

use crate::menu::Lead;

use super::{super::enums::GaugeState, Gauge};

/// A provider's use of its plan: its windows, and a line for what they do not say.
#[derive(Clone, Debug)]
pub struct ProviderGauge {
    pub name: SharedString,
    pub lead: Lead,
    pub gauges: Vec<Gauge>,
    /// What the windows leave out: `$100.03 of $100 extra usage`, `$4.20 of $20 credit`.
    pub note: Option<SharedString>,
    pub state: GaugeState,
}
