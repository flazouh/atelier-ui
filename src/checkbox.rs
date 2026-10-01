//! beui's Checkbox (`components/motion/checkbox.tsx`): a 20px box with a 2px edge and `rounded-md`, and an
//! optional label at `gap-3`. Unchecked, its edge is `muted-foreground` at 50% (full on hover) on the page's
//! fill; checked or partly checked, edge and fill are the primary and the mark is `primary-foreground`. Colours
//! change over 200ms. The mark is a 12px tick (or a dash for a partial choice) drawn with a 1.5px round
//! stroke: it fades and scales in from 50% over 160ms on `EASE_OUT` while the stroke draws itself, 300ms
//! for the tick (200ms for the dash) after a 40ms wait; on the way out it fades and shrinks over 160ms. A
//! press sinks the box to 92% on `SPRING_PRESS`. Keyboard focus draws a 2px ring, 2px outside the box.
//! Disabled is 60%. Under Reduce Motion nothing animates: the mark is there, whole.
//!
//! The web version also blurs the mark by 4px as it leaves; gpui has no blur, so it only fades and shrinks.

mod helpers;
mod structs;
mod types;

pub use helpers::prefix;
pub use structs::Checkbox;
pub use types::{ChangeHandler, Mark};

#[cfg(test)]
use types::{DASH, TICK};

#[cfg(test)]
mod tests;
