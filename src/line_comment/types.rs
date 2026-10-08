use gpui_kit::SharedString;

pub(super) const COMPOSER: &str = "LineComposer";

pub(super) const SEND_KEYS: &str = if cfg!(target_os = "macos") {
    "⌘↵"
} else {
    "⌃↵"
};

/// What a composer reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineComposerEvent {
    /// The user sent `text` for `row`.
    Submit {
        row: usize,
        text: SharedString,
    },
    /// The user put `text` for `row` in the review they are writing, unsent (see [`LineComposer::review_pass`](crate::line_comment::LineComposer::review_pass)).
    Hold {
        row: usize,
        text: SharedString,
    },
    Cancel {
        row: usize,
    },
}
