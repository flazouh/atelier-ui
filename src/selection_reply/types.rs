use gpui_kit::{Pixels, Point, SharedString};

pub(super) const CONTEXT: &str = "SelectionReply";

/// The most characters of the quote the box shows; the whole quote is still reported.
pub(super) const QUOTE_SHOWN: usize = 240;

/// What a selection reply reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionReplyEvent {
    /// The reader replies to `quote` (the words they selected) with `note`, which may be empty. `key` is what
    /// [`SelectionReply::edit`](super::SelectionReply::edit) was given, handed back, when this reply changes an earlier one.
    Reply { quote: SharedString, note: SharedString, key: Option<SharedString> },
}

/// Where the reply is: nothing selected, a button at the end of a selection, or the box open.
pub(super) enum Phase {
    Idle,
    Offer { at: Point<Pixels>, quote: SharedString },
    Writing { at: Point<Pixels>, quote: SharedString, key: Option<SharedString> },
}
