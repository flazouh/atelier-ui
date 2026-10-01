use super::{FoldComposer, MarkBold, MarkCode, MarkItalic, MarkLink, SendComment};

use gpui_kit::{App, KeyBinding};

use super::types::CONTEXT;

/// After the review's keys and the inline review's, so inside the box ⌘B is bold, not the details pane.
pub(crate) fn bind_keys(cx: &mut App) {
    let c = Some(CONTEXT);
    cx.bind_keys([
        KeyBinding::new("secondary-enter", SendComment, c),
        KeyBinding::new("escape", FoldComposer, c),
        KeyBinding::new("secondary-b", MarkBold, c),
        KeyBinding::new("secondary-i", MarkItalic, c),
        KeyBinding::new("secondary-e", MarkCode, c),
        KeyBinding::new("secondary-k", MarkLink, c),
    ]);
}
