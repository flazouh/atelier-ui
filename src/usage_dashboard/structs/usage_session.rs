use gpui_kit::SharedString;

use super::Series;

/// A session's row, and what it shows when it is open.
#[derive(Clone, Debug, PartialEq)]
pub struct UsageSession {
    pub id: SharedString,
    pub title: SharedString,
    /// "atelier · Claude Code · Opus 5.5".
    pub meta: SharedString,
    pub series: Series,
    pub tokens: SharedString,
    pub cost: SharedString,
    /// The tokens of each day of the range, oldest first: the sparkline and the mini chart.
    pub days: Vec<f32>,
    /// Cache read, input and output, each with its value.
    pub split: Vec<(SharedString, SharedString)>,
    /// "46 turns · 3 days": the line under the split, with the cost at its right.
    pub footnote: SharedString,
}
