//! ComboList: the list of beui.dev's Combobox (`components/motion/combobox/list.tsx`). The owner keeps
//! the field, the filter, the keys and the active row; this draws the rows and moves one pill to the
//! active row. The panel around it is the owner's.
//!
//! - List: 6px of padding and at most 256px tall, then it scrolls. Groups have a small heading over
//!   their rows (`0.68rem`, uppercase, `tracking-[0.12em]`).
//! - Row: `rounded-lg`, 8px across and down, 8px between the parts, 14px text. The active row reads in
//!   the foreground and the others muted. One `bg-muted` pill glides to the active row on
//!   `Spring::LAYOUT` (`{ 360, 32, 0.6 }`). A chosen row has a check at the right end that fades in and
//!   grows from 0.82 over 150ms. A disabled row is at 45%.
//! - Empty: "px-3 py-8", centred, muted.
//!
//! The pointer over a row calls [`ComboList::on_hover`], so the owner can make it the active one, as the
//! web does. An active row out of sight is scrolled to. Under Reduce Motion the pill jumps.

mod helpers;
mod structs;
mod types;

pub use helpers::{check_at, scroll_to_show};
pub use structs::{ComboList, ComboRow};
pub use types::{
    ComboEntry, ComboStyle, LINE, MAX_HEIGHT, PAD, PALETTE_SPRING, ROW_GAP, ROW_GAP_BETWEEN,
    ROW_HEIGHT, ROW_PAD_X, ROW_PAD_Y, TEXT,
};

#[cfg(test)]
use crate::motion::Spring;

#[cfg(test)]
mod tests;
