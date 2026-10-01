use std::time::Duration;

/// The stack is at most this wide (`max-w-[22rem]`).
pub(super) const MAX_WIDTH: f32 = 352.;

/// `p-3` round the cards and the footer.
pub(super) const PAD: f32 = 12.;

/// A card behind the first shows this much below it, and is this much narrower on each side, for each place back.
pub(super) const PEEK: f32 = 8.;

pub(super) const INSET: f32 = 12.;

/// The cards in the open list are this far apart (`gap-1`).
pub(super) const GAP: f32 = 4.;

/// Under the cards, when collapsed (`pb-2`), and between the cards and the footer (`mt-2`).
pub(super) const REST_PAD: f32 = 8.;

pub(super) const FOOTER_GAP: f32 = 8.;

/// The footer's height (`min-h-9`), its badge (`size-7`), and its room to the sides (`px-1`).
pub(super) const FOOTER: f32 = 36.;

pub(super) const BADGE: f32 = 28.;

/// A card: `rounded-2xl border px-4`, holding `py-4` of words.
pub(super) const CARD_RADIUS: f32 = 16.;

pub(super) const CARD_PAD_X: f32 = 16.;

pub(super) const CARD_PAD_Y: f32 = 16.;

/// The stack's own corner (`rounded-3xl`).
pub(super) const STACK_RADIUS: f32 = 24.;

/// The time the cards move in, the background's edge, and the old label's exit.
pub(super) const CARDS: f32 = 0.32;

pub(super) const BACKGROUND: f32 = 0.26;

pub(super) const LABEL_EXIT: f32 = 0.14;

/// The labels' line, and how far a rolling label travels.
pub(super) const LABEL_LINE: f32 = 20.;

pub(super) const ROLL: f32 = 0.9;

/// The pointer leaving one part of the stack for another is not leaving the stack: it collapses after this.
pub(super) const LEAVE_GRACE: Duration = Duration::from_millis(40);

/// The tone of the small text at a card's right edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrailingTone {
    Muted,
    Warning,
    Danger,
    Success,
}

pub enum NotificationEvent {
    /// The stack opened or shut.
    Expanded(bool),
    /// A press on the open stack, when it was given [`NotificationStack::view_all`].
    ViewAll,
}
