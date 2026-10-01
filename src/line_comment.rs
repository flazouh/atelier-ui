//! A comment thread on a row of the editor, and its composer. Both sit in a gap below their row (a
//! gpui-base row block, patch 12), so the rows under them move down while they are open.
//!
//! - [`LineComment`]: each comment's initial, author, time, and Markdown body (through
//!   [`AgentText`]), then Reply and Resolve.
//! - [`LineComposer`]: a small multiline input with Comment (primary) and Cancel. `secondary-enter`
//!   sends, Esc cancels. It reports [`LineComposerEvent`]; where the comment goes is the owner's call.
//!
//! Both take the review's card language: a `card_strong` fill, one step above the card the editor sits
//! on, no border, `radius::xl()`, and space around them
//! so the code above and below keeps its rhythm.

gpui_kit::actions!(
    line_comment,
    [
        /// Sends the comment being written.
        SubmitComment,
        /// Drops the comment being written.
        CancelComment,
    ]
);

mod helpers;
mod structs;
mod types;

pub(crate) use helpers::bind_keys;
pub use structs::{Comment, LineComment, LineComposer};
pub use types::LineComposerEvent;

#[cfg(test)]
mod tests;
