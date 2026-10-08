//! The agent's edits shown inside the file the user is editing.
//!
//! Both sides live in the buffer as real text, the way a conflict marker does: the removed rows, then
//! the added rows. Accepting a hunk deletes the removed rows and leaves the agent's; rejecting it does
//! the mirror. The user can type anywhere at any time, and every decision is one undo step, because
//! the edit goes through `EditorState::replace`.
//!
//! See `docs/inline-review.md` for the gpui-base APIs this is built from, and for the two constraints
//! it works around: a row cannot shrink, so a closing hunk leaves a gap that shrinks instead, and a
//! decoration's background is a glyph-run band rather than a row band.
//!
//! Everything in this half of the file is pure. It takes rows and text and returns rows, ranges and
//! offsets, so the whole decision path is tested without a window.

gpui_kit::actions!(
    inline_review,
    [
        /// Accepts the hunk under the caret.
        AcceptHunk,
        /// Rejects the hunk under the caret.
        RejectHunk,
    ]
);

mod helpers;
mod resolve;
mod structs;
mod types;

pub use resolve::Resolve;

pub use helpers::{
    apply, apply_to_text, byte_to_row, compact_bar, hold_caret, hunk_at_row, pending_count,
    plan_edits, row_to_byte, rows_to_bytes, shift_after, track_edit, washes,
};
pub(crate) use helpers::bind_keys;
pub use structs::{DecisionHistory, InlineHunk, InlineReview};
pub use types::{COMPACT_BELOW, Decision};

#[cfg(test)]
use std::ops::Range;

#[cfg(test)]
mod tests;
