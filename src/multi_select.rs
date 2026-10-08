//! beui's MultiSelect (`components/motion/multi-select/`): a field that holds the chosen options as chips, with a
//! text field after them, and a list of options that grows out of it. Typing filters (a subsequence match on the
//! value and the words), the arrow keys wrap round the options that can be chosen, Enter chooses, Backspace on
//! an empty field takes the last chip away, Escape closes. Choosing an option clears the field and keeps the list
//! open.
//!
//! Motion: the list grows to its height on Motion's `{ type: "spring", duration: 0.5, bounce: 0.22 }` and steps
//! 6px away from the field as it does; the active row's wash glides between rows on `SPRING_LAYOUT`; a chip fades
//! in and rises 6px, and leaves by a wipe that shuts it from its left edge in 160ms; the chips after it glide to
//! their new places ([`crate::layout_motion::shifted`]). What gpui cannot draw is left out: the chip's 0.92 scale on
//! entry (text does not scale) and the letter spacing of the group labels is made with gaps.
//!
//! The panel is on the shared [`crate::popover::Popover`] with a hole over the field, so the field stays live while it
//! is open and a press anywhere else only closes it.

mod helpers;
mod structs;
mod types;

pub use helpers::{active, content_height, matches, move_active, visible};
pub use structs::{MultiOption, MultiSelect};
pub use types::MultiSelectEvent;

#[cfg(test)]
use types::{CHIP_HEIGHT, EMPTY, LABEL, ROW};

#[cfg(test)]
use crate::scale::px;
#[cfg(test)]
use gpui_kit::SharedString;

#[cfg(test)]
mod tests;
