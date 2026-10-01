//! A comment on the whole pull request, after GitQuiet's `Saying.tsx` and `Writing.tsx`: "On this pull
//! request", a Markdown box with Write and Preview and a small toolbar (bold, italic, code, link, quote,
//! list), and Comment and Cancel.
//!
//! - The toolbar writes Markdown around the chosen words ([`crate::markdown_edit`]). Command with B, I, E
//!   and K do the same as the first four, as in every editor; each tooltip says so.
//! - `secondary-enter` sends. Escape and Cancel fold the box and keep the words: Cancel is not a word
//!   that means delete, anywhere else people write. A folded box with words in it says "Carry on with what
//!   you were writing".
//! - It reports [`CommentComposerEvent`]; where the comment goes is the owner's call.

gpui_kit::actions!(
    comment_composer,
    [
        /// Sends the comment.
        SendComment,
        /// Folds the box, keeping the words.
        FoldComposer,
        MarkBold,
        MarkItalic,
        MarkCode,
        MarkLink,
    ]
);

mod helpers;
mod structs;
mod types;

pub(crate) use helpers::bind_keys;
pub use structs::CommentComposer;
pub use types::CommentComposerEvent;
