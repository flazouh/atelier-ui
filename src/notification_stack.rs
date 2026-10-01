//! beui's NotificationStack (`components/motion/notification-stack.tsx`): notifications as a small stack of cards that
//! opens into a list. Collapsed, the newest card is whole and the ones behind it peek out below it, each 8px lower
//! and 12px narrower on each side; a count badge and "Notifications" sit under them. Opened, by a pointer over it, focus,
//! or a tap, the cards spread into a list, one to a row 4px apart, and the badge's words roll to "View all ↗". The stack
//! grows upward from where it sat, so it covers what is above it and moves nothing.
//!
//! Motion: the cards move on 320ms of the out curve, the background's edge on 260ms of it, and the label rolls on
//! `SPRING_SWAP` (the old words leave up in 140ms). Under Reduce Motion it jumps. A card behind the first has no words
//! until the stack is open. What gpui cannot draw is left out: the blur on the label.
//!
//! The geometry is worked out here, from the cards' heights measured in a hidden column, so what is drawn is a plain
//! function of how far open the stack is ([`Geometry`]).

mod helpers;
mod structs;
mod types;

pub use structs::{Geometry, NotificationItem, NotificationStack, Trailing};
pub use types::{NotificationEvent, TrailingTone};

#[cfg(test)]
mod tests;
