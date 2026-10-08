use std::{rc::Rc, sync::Arc};

use gpui_kit::{App, SharedString, Window};

/// What pressing a row's "add a comment" button reports: the row.
pub(super) type RowHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// What a decision on one hunk is reported with.
pub(super) type DecideHandler = Arc<dyn Fn(&SharedString, Decision, &mut Window, &mut App)>;

/// What the user chose for one hunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Keep the agent's rows and drop the old ones.
    Accept,
    /// Keep the old rows and drop the agent's.
    Reject,
}

/// The keys the bar names, as each platform writes them.
pub(super) const ACCEPT_KEYS: &str = if cfg!(target_os = "macos") {
    "⌘↵"
} else {
    "⌃↵"
};

pub(super) const REJECT_KEYS: &str = if cfg!(target_os = "macos") {
    "⌘⌫"
} else {
    "⌃⌫"
};

/// Below this width the hunk bar keeps its two icons and drops the words and the key caps, so it does not
/// cover the code it decides. The words and keys move to the tooltips.
pub const COMPACT_BELOW: f32 = 640.;
