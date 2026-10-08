use gpui_kit::SharedString;

use super::Series;

/// A model's share of the range.
#[derive(Clone, Debug, PartialEq)]
pub struct UsageModel {
    pub name: SharedString,
    pub tokens: u64,
    pub series: Series,
    /// The tokens as the row says them: "41.2 M".
    pub label: SharedString,
}
