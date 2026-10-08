use gpui_kit::SharedString;

use super::Series;

/// One day of the chart: its label, the line that tells it, and what each series used, the lowest in the bar first.
#[derive(Clone, Debug, PartialEq)]
pub struct UsageDay {
    /// The day number: "9".
    pub label: SharedString,
    /// What the line above the chart says when the day is highlighted: "Thu 9 · 5.84 M tokens · $18.40".
    pub detail: SharedString,
    pub parts: Vec<(Series, u64)>,
}

impl UsageDay {
    pub fn total(&self) -> u64 {
        self.parts.iter().map(|(_, tokens)| tokens).sum()
    }
}
