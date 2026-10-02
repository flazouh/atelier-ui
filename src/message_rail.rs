//! beui's PreviewRail as the conversation's rail (`components/motion/preview-rail.tsx`, used by `message-scroller` with
//! `navigation="rail"`): a column of short ticks at the list's edge, one for each message you sent. The tick under the
//! pointer, or the one for the message in view, is full length; its neighbours are 68%, 44% and 25% of it, so the
//! column swells like a dock, each tick on `Spring::LAYOUT`. The pointer on a tick shows a card beside it with the
//! message's start and the start of the answer, which rises 4px and fades in over 180ms; a press scrolls to that message.

mod helpers;
mod structs;
mod types;

pub use helpers::{excerpt, item_size, tick_scale};
pub use structs::{MessageRail, RailItem};
pub use types::{DESCRIPTION_CHARS, LABEL_CHARS};

#[cfg(test)]
mod tests;
