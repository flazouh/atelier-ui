//! Digits: beui.dev's Digit Swap (`components/motion/digit-swap.tsx`), from Number Animation. A number in fixed
//! slots: when a digit changes, the old one leaves through the top and the new one comes up from 45% below,
//! clear to full over 180ms ([`crate::roll::Kind::Digit`]). A digit that did not change does not move.
//!
//! [`Digits`] takes any text and gives every run of digits its slots, one per character, `1ch` wide (0.6em in
//! the monospace font, the same in the text font, whose digits differ little), as tall as the line of the text around them and clipped. The
//! rest of the text is plain, so "3 of 12 reviewed" rolls its two numbers and keeps its words still.
//!
//! Under Reduce Motion a digit changes at once. What gpui cannot draw is left out: the 6ms stagger from one
//! slot to the next (a slot keeps its place, not its delay).

mod helpers;
mod structs;
mod types;

pub use helpers::{line_for, runs};
pub use structs::Digits;
pub use types::SLOT_WIDTH;

#[cfg(test)]
mod tests;
