use gpui_kit::{Pixels, Point, SharedString};

use crate::voice_input::VoiceInputEvent;

pub(super) const CONTEXT: &str = "SelectionReply";
/// How long the bar takes to settle in: a short rise from half strength, so it is there at once and still has a landing.
pub(super) const OFFER_IN: std::time::Duration = std::time::Duration::from_millis(90);

/// The most characters of the quote the box shows; the whole quote is still reported.
pub(super) const QUOTE_SHOWN: usize = 240;

/// What a selection reply reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionReplyEvent {
    /// The reader replies to `quote` (the words they selected) with `note`, which may be empty. `key` is what
    /// [`SelectionReply::edit`](super::SelectionReply::edit) was given, handed back, when this reply changes an earlier one.
    Reply { quote: SharedString, note: SharedString, key: Option<SharedString> },
    /// The reader pressed the microphone (see [`SelectionReply::dictation`](super::SelectionReply::dictation)): the
    /// owner starts or stops listening, and hands the words back with
    /// [`SelectionReply::insert_transcript`](super::SelectionReply::insert_transcript).
    Dictate(VoiceInputEvent),
    /// The box closed while the microphone listened: the owner drops that press.
    DictationCancel,
}

/// Where the reply is: nothing selected, a button at the end of a selection, or the box open.
pub(super) enum Phase {
    Idle,
    Offer { at: Point<Pixels>, quote: SharedString },
    Writing { at: Point<Pixels>, quote: SharedString, key: Option<SharedString> },
}

/// A one-press reply on the offer: its `label` is on the button, and its `note` goes as the reply's note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplyPreset {
    pub label: SharedString,
    pub note: SharedString,
}
impl ReplyPreset {
    pub fn new(label: impl Into<SharedString>, note: impl Into<SharedString>) -> Self {
        Self { label: label.into(), note: note.into() }
    }
}
