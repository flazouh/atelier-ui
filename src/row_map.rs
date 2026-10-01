//! The rows an inline review shows against the rows of the file itself. The review's buffer holds each
//! hunk's removed rows above its added ones, so a language server, which reads the file as it is at the
//! pull request's head, must be given the text without the removed rows and have every row mapped
//! across: a removed row has no row in the file.

use std::ops::Range;

use crate::inline_review::InlineHunk;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RowMap {
    /// The shown rows that were removed, in order.
    removed: Vec<Range<usize>>,
}

impl RowMap {
    /// The map for a buffer showing `hunks`. With no hunks, every row is its own.
    pub fn new(hunks: &[InlineHunk]) -> Self {
        let mut removed: Vec<Range<usize>> = hunks.iter().map(|h| h.removed.clone()).filter(|r| !r.is_empty()).collect();
        removed.sort_by_key(|r| r.start);
        Self { removed }
    }

    /// The file's text: `shown` without its removed rows.
    pub fn head_text(&self, shown: &str) -> String {
        shown
            .split('\n')
            .enumerate()
            .filter(|(row, _)| !self.is_removed(*row))
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The file's row for a shown row, or `None` for a removed one.
    pub fn to_head(&self, row: usize) -> Option<usize> {
        if self.is_removed(row) {
            return None;
        }
        Some(row - self.removed.iter().filter(|r| r.end <= row).map(|r| r.len()).sum::<usize>())
    }

    /// The shown row for a row of the file.
    pub fn to_view(&self, head: usize) -> usize {
        let mut row = head;
        for removed in &self.removed {
            if removed.start > row {
                break;
            }
            row += removed.len();
        }
        row
    }

    fn is_removed(&self, row: usize) -> bool {
        self.removed.iter().any(|r| r.contains(&row))
    }
}

#[cfg(test)]
mod tests;
