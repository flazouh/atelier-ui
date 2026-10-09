use gpui_kit::SharedString;

/// One figure of the row under the title: its label, the big value with its unit, a note under it, and a meter when the
/// figure is a share of a limit.
#[derive(Clone, Debug, PartialEq)]
pub struct UsageStat {
    pub label: SharedString,
    pub value: SharedString,
    /// "tokens", "used", "14 days".
    pub unit: SharedString,
    pub note: SharedString,
    /// The fraction used, 0 to 1, for a meter under the value.
    pub used: Option<f32>,
}
