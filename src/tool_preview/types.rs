use gpui_kit::SharedString;

use crate::file_diff::{DiffLine, DiffLineKind};
use super::structs::TextEdit;
use super::helpers::line_diff;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolPreview {
    /// Edits to one file (an Edit is one, a MultiEdit is several), under the file's path. `start_line` is where the
    /// first edit's text begins in the file, when the caller knows; else 1.
    Edits { path: SharedString, edits: Vec<TextEdit>, start_line: u32 },
    /// A file written whole: every line is new.
    Written { path: SharedString, text: SharedString },
    /// A shell command.
    Command { text: SharedString },
}

impl ToolPreview {
    pub fn edit(path: impl Into<SharedString>, old: impl Into<SharedString>, new: impl Into<SharedString>) -> Self {
        Self::Edits { path: path.into(), edits: vec![TextEdit::new(old, new)], start_line: 1 }
    }

    pub fn edits(path: impl Into<SharedString>, edits: Vec<TextEdit>) -> Self {
        Self::Edits { path: path.into(), edits, start_line: 1 }
    }

    pub fn written(path: impl Into<SharedString>, text: impl Into<SharedString>) -> Self {
        Self::Written { path: path.into(), text: text.into() }
    }

    pub fn command(text: impl Into<SharedString>) -> Self {
        Self::Command { text: text.into() }
    }

    /// The file's path, for a preview of a file.
    /// The command a Command preview shows.
    pub fn command_text(&self) -> Option<&SharedString> {
        match self {
            ToolPreview::Command { text } => Some(text),
            _ => None,
        }
    }

    pub fn path(&self) -> Option<&SharedString> {
        match self {
            Self::Edits { path, .. } | Self::Written { path, .. } => Some(path),
            Self::Command { .. } => None,
        }
    }

    /// The rows of the diff; none for a command.
    pub fn rows(&self) -> Vec<DiffLine> {
        match self {
            Self::Edits { edits, start_line, .. } => {
                let mut rows = Vec::new();
                for (i, edit) in edits.iter().enumerate() {
                    if edits.len() > 1 {
                        rows.push(DiffLine { kind: DiffLineKind::Hunk, old_line: None, new_line: None, text: format!("@@ change {} of {} @@", i + 1, edits.len()).into() });
                    }
                    rows.extend(line_diff(&edit.old, &edit.new, *start_line));
                }
                rows
            }
            Self::Written { text, .. } => line_diff("", text, 1),
            Self::Command { .. } => Vec::new(),
        }
    }
}

/// The most cells of the line table before a diff falls back to "all removed, all added".
pub(super) const TABLE_LIMIT: usize = 4_000_000;
