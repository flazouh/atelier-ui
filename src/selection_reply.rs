//! Reply to words in the conversation. The reader selects text anywhere under the window's text selection
//! (messages, command output, diffs); a small "Reply" button floats at the end of the selection, and a press
//! opens a compact box: the words quoted on one line, a note, a microphone (when the owner turns it on with
//! [`SelectionReply::dictation`]) and a round Add button. It reports
//! [`SelectionReplyEvent::Reply`]; where the quote and the note go is the owner's call.
//!
//! - It reads the window's own selection ([`TextSelection`](gpui_kit::base::TextSelection)), so it needs no
//!   knowledge of what was selected or where: whatever the reader can select, they can reply to.
//! - It draws nothing until a selection is released. Make it a child of the (`relative`) container whose words may
//!   be replied to, after that content (it reads the selection as it paints, once the words have painted theirs): it fills the
//!   container but takes no hits, and a selection that ends outside it is not offered.
//! - [`SelectionReply::edit`] opens the box again on an earlier reply.
//! - Enter adds the reply; Shift-Enter makes a new line; Escape drops it. A reply with no note is
//!   allowed: the quote alone is a message to the agent.

gpui_kit::actions!(
    selection_reply,
    [
        /// Adds the reply being written.
        AddReply,
        /// Drops the reply being written.
        DropReply,
    ]
);

mod helpers;
mod structs;
mod types;

pub(crate) use helpers::bind_keys;
pub use structs::SelectionReply;
pub use types::SelectionReplyEvent;

#[cfg(test)]
mod tests;
