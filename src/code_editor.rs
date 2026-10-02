//! The editable code surface: gpui-base's code editor, skinned for atelier.
//!
//! gpui-base owns the hard parts, so this file does not reimplement any of them: editing, selection,
//! undo, IME, search, folding, line numbers, and tree-sitter highlighting all come from
//! [`gpui_kit::component::input::EditorState`]. atelier supplies the look and the language list.
//!
//! - Geist Mono at `text-xs`, 20px rows, the gutter in `muted_foreground` at 40%, the current line
//!   washed with `card`.
//! - Token colors come from the theme in force (`assets/themes`); atelier's are the muted palette: keywords take
//!   `info`, strings `success`, numbers and escapes `warning`, comments muted, types the ramp's light
//!   tone. Nothing here uses Tailwind's chroma.
//! - Diagnostics underline their range in `danger` for an error and `warning` for a warning. The caller
//!   pushes them; `crate::lsp` is what fills them in.
//! - `read_only` is for a file with a hunk review open, so the two never fight over one buffer.
//! - Four-space indents, and the line commands in [`commands`] on Zed's keys: toggle comment, move
//!   line, duplicate line, delete line, a Home that stops at the indent, and copy, cut or paste of the
//!   whole line when nothing is selected.

gpui_kit::actions!(
    code_editor,
    [
        /// Comments out the selected lines, or uncomments them.
        ToggleComment,
        /// Swaps the selected lines with the line above.
        MoveLineUp,
        /// Swaps the selected lines with the line below.
        MoveLineDown,
        /// Copies the selected lines below themselves.
        DuplicateLine,
        /// Removes the selected lines.
        DeleteLine,
        /// Home that stops at the indent first.
        SmartHome,
        /// Copy, or the whole line when nothing is selected.
        CopyLine,
        /// Cut, or the whole line when nothing is selected.
        CutLine,
        /// Paste, and a whole line copied that way goes above the caret's line.
        PasteLine,
        /// Lists every use of the symbol at the caret. The editor only names the key; whoever owns the
        /// language server answers it.
        FindReferences,
    ]
);

pub mod commands;
mod helpers;
mod structs;
mod types;

pub use crate::syntax::{LANGUAGES, language_for};

pub use helpers::{install_syntax_theme, set_diagnostics, severity_tone, syntax_theme};
pub(crate) use helpers::{bind_keys, surface};
pub use structs::CodeEditor;
pub use types::ROW_HEIGHT;

#[cfg(test)]
use gpui_kit::base::input::DiagnosticSeverity;
#[cfg(test)]
use crate::theme::Appearance;

#[cfg(test)]
mod tests;
