use gpui_kit::{AnyElement, SharedString};

use crate::icon::IconName;

/// How a button of the head looks: its name, icon and tip, whether it marks a choice in force (`lit`), whether its menu
/// is open, and the menu itself.
pub(super) struct HeadButton {
    pub(super) id: &'static str,
    pub(super) icon: IconName,
    pub(super) tip: SharedString,
    pub(super) lit: bool,
    pub(super) open: bool,
    pub(super) menu: Option<AnyElement>,
}
