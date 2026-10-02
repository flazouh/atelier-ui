//! beui's FileDiff (`components/agents/file-diff.tsx`), class for class:
//!
//! - A card, as in [`crate::subagent_card::SubagentCard`]: `bg-card rounded-2xl`, the header as compact as a strip row (`h-8 px-3`), the diff in a
//!   darker well that runs to the card's edges.
//! - Root `w-full text-sm`. Header `min-h-9 gap-2`: a `size-4` file icon, the path
//!   `text-xs` at 80% foreground, `+n`/`\u{2212}n` change counts, a `size-4` status slot (spinning loader
//!   while streaming, a check once complete), and a rotating `size-3.5` chevron.
//! - Body `pl-6 pt-1.5`: a `rounded-xl` card holding a scrollable two-column line-number gutter (old,
//!   new), a `size-4` sign column, and the line text, then a footer row with Copy when `copy_text` is set.
//! - Rows are a virtual list (`uniform_list`): each is [`ROW_HEIGHT`] tall, and only the rows in view
//!   are built and laid out, so a 5k-row diff costs a frame what a short one does.
//! - It opens while streaming and closes by itself on completion, like beui's `collapseOnComplete`.
//! - While streaming, the rows follow their newest line, so an edit being written reads like a terminal; a finished diff stays where
//!   the reader left it.
//! - Syntax colours as in the editor ([`crate::syntax`]): each side of the diff is highlighted as one whole
//!   text, off the UI thread, cached, and each row reads its side's line. The +/− washes sit under the colours.

mod helpers;
mod structs;
mod types;

pub use helpers::diff_stats;
pub use structs::{DiffLine, FileDiff};
pub use types::{DiffLineKind, FileDiffStatus, ROW_HEIGHT};

#[cfg(test)]
mod tests;
