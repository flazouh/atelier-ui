//! Tabs: beui.dev's Tabs (`components/motion/tabs.tsx`). A row of tabs with one indicator that glides from
//! the chosen tab to the next on `{ stiffness: 245, damping: 36, mass: 1.2 }`, which settles without
//! overshoot. Three looks:
//!
//! - Pill: `rounded-full bg-card p-1`, tabs with `px-3.5 py-1.5`, and a primary pill under the chosen one.
//! - Segment: `rounded-lg bg-card p-0.5`, no gap, and a `rounded-md` primary pill.
//! - Underline: tabs of at least 44px (`px-3 pb-2.5 pt-1`) over a 1px border, and a 1px primary line
//!   under the chosen one.
//!
//! The words are muted and go to the foreground on hover. On a pill they are the primary's text colour
//! by how much of the tab the pill covers, so the change follows the glide. Under Reduce Motion the
//! indicator jumps. A tab may carry something before its words and something after (a file's icon, a
//! close button); a pending tab (a file still being read) has no indicator and cannot be chosen.
//!
//! What gpui cannot draw is left out: the scroll arrows and edge fades of a list too long for its room
//! (the list scrolls), and the content's 4px rise on a change.

mod helpers;
mod structs;
mod types;

pub use helpers::covered;
pub use structs::{Tab, Tabs};
pub use types::{EDITOR_HEIGHT, GLIDE, TabsVariant, UNDERLINE_HEIGHT};

#[cfg(test)]
use crate::motion::Animated;
#[cfg(test)]
use gpui_kit::Pixels;

#[cfg(test)]
mod tests;
