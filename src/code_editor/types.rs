/// The key context around every [`CodeEditor`](crate::code_editor::CodeEditor), so these keys never reach a one-line input.
pub(super) const CONTEXT: &str = "CodeEditor > Input";

/// Marks a clipboard entry as one whole line, so a paste knows to put it above the caret's line.
pub(super) const WHOLE_LINE: &str = "atelier.whole-line";

/// Row height, matching the hunk review so a diff and its file line up.
pub const ROW_HEIGHT: f32 = 20.;
