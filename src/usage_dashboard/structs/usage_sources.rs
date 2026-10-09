use gpui_kit::{ElementId, IntoElement, SharedString};

use super::{UsageSource, usage_dashboard::OnSelect};
use crate::usage_dashboard::Selection;

/// The list of accounts and keys, for a sidebar: "All accounts" first, then each provider's under its caption, each with its
/// meter. A press picks one, or a whole provider by its caption. It is stateless: the app holds the selection.
#[derive(IntoElement)]
pub struct UsageSources {
    pub(in super::super) id: ElementId,
    pub(in super::super) sources: Vec<UsageSource>,
    pub(in super::super) summary: Option<(SharedString, SharedString)>,
    pub(in super::super) selection: Selection,
    pub(in super::super) on_select: Option<OnSelect>,
}
