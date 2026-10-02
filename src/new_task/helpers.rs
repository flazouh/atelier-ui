use super::SubmitTask;

/// ⌘↵ creates the task from any field of the dialog: the description takes Enter itself, so the key is bound here,
/// deeper than its own.
pub(crate) fn bind_keys(cx: &mut gpui_kit::App) {
    cx.bind_keys([gpui_kit::KeyBinding::new("secondary-enter", SubmitTask, Some("NewTask > Input"))]);
}
