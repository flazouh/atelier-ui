use gpui_kit::SharedString;

pub(super) const CONTEXT: &str = "CommentComposer > Input";

pub(super) const SEND: &str = if cfg!(target_os = "macos") {
    "⌘↵"
} else {
    "⌃↵"
};

pub(super) const MOD: &str = if cfg!(target_os = "macos") {
    "⌘"
} else {
    "⌃"
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommentComposerEvent {
    Submit(SharedString),
}
