use gpui_kit::SharedString;

use super::Series;
use crate::usage_dashboard::SourceKind;

/// An account or a key: one tile.
#[derive(Clone, Debug, PartialEq)]
pub struct UsageSource {
    pub id: SharedString,
    pub name: SharedString,
    /// Under the name: "personal", "key".
    pub caption: SharedString,
    /// The provider's caption over its tiles: "Claude Code · 3 accounts". Sources that follow each other with the same
    /// group stand under one caption.
    pub group: SharedString,
    pub series: Series,
    /// The fraction used, 0 to 1; none for a key with no limit, which has no gauge.
    pub limit: Option<f32>,
    /// "45%", "$4.20".
    pub value: SharedString,
    /// "7d", "of $20".
    pub note: SharedString,
    pub kind: SourceKind,
}
