use gpui_kit::SharedString;

/// One window of a provider's limit: the five hours, the week, the month.
#[derive(Clone, Debug, PartialEq)]
pub struct Gauge {
    /// The window, short: `5h`, `7d`, `30d`.
    pub label: SharedString,
    /// How much of the window is used, 0 to 1.
    pub used: f32,
    /// Seconds until the window starts again, when the provider says.
    pub resets_in: Option<u64>,
}
