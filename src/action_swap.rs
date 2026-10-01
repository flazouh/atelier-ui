//! ActionSwapButton: beui.dev's Action Swap (`components/motion/action-swap.tsx`). One button whose words
//! swap for the next step's as the steps complete ("Commit", then "Push", then "Open pull request"): the
//! old words roll up and out through the top, and the new come up from below ([`crate::roll::Kind::Swap`]).
//!
//! - Small: `h-8`, `rounded-full`, `px-3`, 12px medium words with 6px between the parts.
//! - Primary: the primary fill and its text, a tenth lighter toward the page on hover. Secondary is the card with a
//!   border; Ghost is muted words that take the foreground on hover.
//! - A key cap follows the words: a command's, read from the key table, or one given.
//! - A press squeezes it to 97%; it does here as a 1.5% pull-in of each side.
//!
//! What gpui cannot draw is left out: the blur on the words as they move.

mod helpers;
mod structs;
mod types;

pub use helpers::colors;
pub use structs::ActionSwapButton;
pub use types::{GAP, HEIGHT, LINE, PAD_X, PRESS_INSET, SwapSize, SwapVariant, TEXT};

#[cfg(test)]
mod tests;
