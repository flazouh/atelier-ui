//! Syntax colours for code outside the editor: diffs, code blocks and the pull request view, drawn with the
//! editor's own highlighter (gpui-component's tree-sitter `SyntaxHighlighter`) and the editor's colours
//! (the theme in force), so a line looks the same in all of them.
//!
//! - A text is parsed as a whole, never line by line, so a string or a comment across lines comes out
//!   right. A diff parses each of its two sides as one text and maps each row to its side's line.
//! - Each text is parsed once. Its styles, split into lines, are cached by language, content and theme
//!   ([`SyntaxCache`]), so scrolling, hover and a re-render never parse again.
//! - Every text parses on a background thread, never on the UI thread: even a short one can cost more
//!   than a frame when it needs a query built ("Performance" in `docs/code-editor.md`).
//! - Each background thread keeps one highlighter per language: building one compiles its queries,
//!   which costs far more than parsing a code block.
//! - While a text parses, its slot (a code block, a diff's side) keeps the colours of its last text on
//!   each row that did not change, so a streamed block does not flash plain at each token. A new row
//!   draws plain until the parse lands. The colours change nothing else, so the rows do not move.
//! - A slot's new text drops the parse of its old one: the background job checks the slot's
//!   generation before it starts and before it writes, and stops when a newer text replaced it.

mod helpers;
mod structs;
mod types;

pub use helpers::{carry, compute, compute_with, highlight, language_for, row_sides, sides};
pub use structs::{Key, SideText, SyntaxCache};
pub use types::{LANGUAGES, LineRuns, MAX_ENTRIES, Side};

#[cfg(test)]
use gpui_kit::{ElementId, component::highlighter::SyntaxHighlighter};
#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
mod tests;
