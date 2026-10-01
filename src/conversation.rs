//! Everything said on a pull request, folded to one line each, after GitQuiet's `Conversation.tsx`:
//! threads first, the open ones leading, then Remarks. A thread's line has up to three faces, its first
//! words, and how many comments it holds; a resolved one recedes to 60% with a tick instead of the fold
//! mark, and stays, since it is the record of why the code looks as it does. A Remark has one face, its
//! first words, and no count: nobody owes it an answer. Pressing a line opens its comments, as a
//! [`crate::line_comment::LineComment`].
//!
//! The header counts what is still open first: "4 open, 1 resolved, 3 remarks".

mod helpers;
mod structs;
mod types;

pub use helpers::{faces, open_first, said_by_count, said_so_far};
pub use structs::{ConversationList, RemarkSummary, ThreadSummary};
pub use types::{MoreHandler, ThreadHandler};

#[cfg(test)]
mod tests;
