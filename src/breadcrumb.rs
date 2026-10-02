//! Breadcrumb: beui.dev's Animated Breadcrumb (`components/motion/breadcrumb.tsx`). The path to where the reader is,
//! each part a link to the level it names and the last one the page it is on.
//!
//! - Link: atelier's numbers from the editor header: 28px tall, `rounded-md`, 10px across, 14px medium words, muted, going to the foreground on a
//!   `bg-muted/60` wash under the pointer. The page is the same box in the foreground.
//! - Separator: a chevron, 14px, at 50% of the muted tone, before every part but the first.
//! - Overflow: with more than `max_items` (4 by default, at least 3) the first part stays, the middle ones fold into
//!   an ellipsis button that opens a menu of them, and the last `max_items - 2` stay.
//! - A part that arrives after the first frame comes in over 200ms from 6px below and clear; the parts
//!   already there stay. Under Reduce Motion it appears at once.
//!
//! What gpui cannot draw is left out: the parts sliding to their places on the layout spring and the exit of
//! a part that leaves (it goes at once), and the hover-open of the ellipsis (a press opens it).

mod helpers;
mod structs;
mod types;

pub use helpers::{hidden, shown};
pub use structs::{Breadcrumb, Crumb};
pub use types::{DEFAULT_SHOWN, ENTER_Y, HEIGHT, ICON, LINK_PAD, MIN_SHOWN};

#[cfg(test)]
mod tests;
