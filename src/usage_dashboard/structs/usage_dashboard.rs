use std::rc::Rc;

use gpui_kit::{App, ElementId, IntoElement, SharedString, Window};

use super::{UsageDay, UsageModel, UsageSession, UsageSource};
use crate::usage_dashboard::{Selection, UsageRange};

pub(in super::super) type OnRange = Rc<dyn Fn(UsageRange, &mut Window, &mut App)>;
pub(in super::super) type OnSelect = Rc<dyn Fn(Selection, &mut Window, &mut App)>;
pub(in super::super) type OnExpand = Rc<dyn Fn(Option<SharedString>, &mut Window, &mut App)>;

/// The usage dashboard.
#[derive(IntoElement)]
pub struct UsageDashboard {
    pub(in super::super) id: ElementId,
    pub(in super::super) range: UsageRange,
    pub(in super::super) selection: Selection,
    pub(in super::super) sources: Vec<UsageSource>,
    pub(in super::super) summary: Option<(SharedString, SharedString)>,
    pub(in super::super) days: Vec<UsageDay>,
    pub(in super::super) models: Vec<UsageModel>,
    pub(in super::super) sessions: Vec<UsageSession>,
    pub(in super::super) expanded: Option<SharedString>,
    pub(in super::super) total: Option<SharedString>,
    pub(in super::super) empty: Option<SharedString>,
    pub(in super::super) on_range: Option<OnRange>,
    pub(in super::super) on_select: Option<OnSelect>,
    pub(in super::super) on_expand: Option<OnExpand>,
}
