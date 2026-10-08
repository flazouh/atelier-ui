//! Reply to words in the conversation. The reader selects text anywhere under the window's text selection (messages,
//! command output, diffs), and a small box opens at the end of the selection in the same frame, with its note already
//! focused, so they can type at once. It holds the quote in a message bubble, one-press badges
//! ([`SelectionReply::presets`]: a label, an icon and a colour each), the note, a microphone (when the owner turns it
//! on with [`SelectionReply::dictation`]) and a round Add button. It reports [`SelectionReplyEvent::Reply`]; where the
//! quote and the note go is the owner's call.
//!
//! - It reads the window's own selection ([`TextSelection`](gpui_kit::base::TextSelection)), so it needs no
//!   knowledge of what was selected or where: whatever the reader can select, they can reply to.
//! - It draws nothing until a selection is released. Make it a child of the (`relative`) container whose words may
//!   be replied to, after that content (it reads the selection as it paints, once the words have painted theirs): it
//!   fills the container but takes no hits, and a selection that ends outside it is not offered.
//! - [`SelectionReply::edit`] opens the box again on an earlier reply.
//! - Enter adds the reply; Shift-Enter makes a new line; Escape drops it, and the selection with it. A press outside
//!   the box closes it and leaves the selection alone. A reply with no note is allowed: the quote alone is a message to
//!   the agent.
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
pub use types::{ReplyPreset, SelectionReplyEvent};

#[cfg(test)]
mod tests;
