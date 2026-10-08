use gpui_kit::SharedString;

use crate::file_diff::{DiffLine, DiffLineKind};
use super::types::TABLE_LIMIT;

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
