//! The line commands a Zed user reaches for, which gpui-base does not ship: toggle a comment, move
//! lines up and down, duplicate or delete them, a Home that stops at the indent first, and copy or cut
//! of the whole line when nothing is selected.
//!
//! Every command works on all the selections at once. It takes the text and the selections, and
//! returns one [`Edit`]: a single replacement and the selections to put back, so a multi-cursor
//! command is one undo step and keeps every cursor. Each is a pure function, tested without a window.
//! [`crate::code_editor::CodeEditor`] binds the keys. Offsets are bytes, as gpui-base's are.
// A selection list with one range is the common case here, not a typo for the range itself.
#![allow(clippy::single_range_in_vec_init)]

mod helpers;
mod structs;

pub use helpers::{
    comment_prefix, delete_lines, duplicate_lines, line_span, move_lines, smart_home,
    toggle_comment, whole_line_paste, whole_lines,
};
pub use structs::Edit;

#[cfg(test)]
use std::ops::Range;

#[cfg(test)]
mod tests;
