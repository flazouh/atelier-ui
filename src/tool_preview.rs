//! What a tool approval shows the reader instead of the tool's raw input: an edit as a diff under its path, a file
//! written as all-new lines, a shell call as its command. It takes plain data, so the app maps a tool call to
//! one of these and atelier-ui knows nothing of any agent. The raw input stays behind "View details".
use gpui_kit::SharedString;

use crate::file_diff::{DiffLine, DiffLineKind};

/// One replacement: the text that was there, and the text that goes in its place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextEdit {
    pub old: SharedString,
    pub new: SharedString,
}

impl TextEdit {
    pub fn new(old: impl Into<SharedString>, new: impl Into<SharedString>) -> Self {
        Self { old: old.into(), new: new.into() }
    }
}

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
const TABLE_LIMIT: usize = 4_000_000;

fn lines_of(text: &str) -> Vec<&str> {
    if text.is_empty() { Vec::new() } else { text.strip_suffix('\n').unwrap_or(text).split('\n').collect() }
}

/// The line diff of `old` into `new`, every line of both in order, numbered from `start`. It is the longest common
/// run of lines, so the lines an edit keeps read as context.
pub fn line_diff(old: &str, new: &str, start: u32) -> Vec<DiffLine> {
    let (a, b) = (lines_of(old), lines_of(new));
    let (n, m) = (a.len(), b.len());
    let mut rows = Vec::with_capacity(n + m);
    let (mut old_no, mut new_no) = (start.saturating_sub(1), start.saturating_sub(1));
    let mut push = |kind: DiffLineKind, text: &str, rows: &mut Vec<DiffLine>| {
        let (old_line, new_line) = match kind {
            DiffLineKind::Context => {
                old_no += 1;
                new_no += 1;
                (Some(old_no), Some(new_no))
            }
            DiffLineKind::Removed => {
                old_no += 1;
                (Some(old_no), None)
            }
            _ => {
                new_no += 1;
                (None, Some(new_no))
            }
        };
        rows.push(DiffLine { kind, old_line, new_line, text: text.to_string().into() });
    };
    if (n + 1) * (m + 1) > TABLE_LIMIT {
        a.iter().for_each(|l| push(DiffLineKind::Removed, l, &mut rows));
        b.iter().for_each(|l| push(DiffLineKind::Added, l, &mut rows));
        return rows;
    }
    // table[i][j]: the common run of a[i..] and b[j..].
    let mut table = vec![0u32; (n + 1) * (m + 1)];
    let at = |i: usize, j: usize| i * (m + 1) + j;
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            table[at(i, j)] = if a[i] == b[j] { table[at(i + 1, j + 1)] + 1 } else { table[at(i + 1, j)].max(table[at(i, j + 1)]) };
        }
    }
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            push(DiffLineKind::Context, a[i], &mut rows);
            i += 1;
            j += 1;
        } else if i < n && (j == m || table[at(i + 1, j)] >= table[at(i, j + 1)]) {
            push(DiffLineKind::Removed, a[i], &mut rows);
            i += 1;
        } else {
            push(DiffLineKind::Added, b[j], &mut rows);
            j += 1;
        }
    }
    rows
}

/// `path` relative to `root`, when it is inside it; else `path` as it is.
pub fn relative_path(path: &str, root: &str) -> SharedString {
    let root = root.trim_end_matches('/');
    match path.strip_prefix(root).and_then(|rest| rest.strip_prefix('/')) {
        Some(rest) if !root.is_empty() && !rest.is_empty() => rest.to_string().into(),
        _ => path.to_string().into(),
    }
}

#[cfg(test)]
mod tests;
