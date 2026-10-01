//! Roll: the swap of beui.dev's Animated Badge (`components/motion/animated-badge.tsx`), for anything that
//! changes in place. When its `key` changes, the old content leaves upward and the new content comes up from
//! below into its place, inside a box that clips both.
//!
//! - Enter: from 80% (85% for words) of the content's height below, at 72% (76%) opacity, to rest. The rise is
//!   a spring `{ stiffness: 210, damping: 24, mass: 0.85 }`; the opacity takes 280ms (300ms for words).
//! - Exit: to 80% (85%) of the height above, to 50% opacity, over 220ms (200ms for words). The old content
//!   leaves the flow at once, so the box takes the new content's size.
//! - Under Reduce Motion the content changes at once.
//!
//! What gpui cannot draw is left out: the blur (6px), the 0.92 scale and the 8 degrees of turn of an icon.
//! The content is drawn by the `build` function, once for what is here and once more for what is leaving.

mod helpers;
mod structs;
mod types;

pub use helpers::{entering, leaving};
pub use structs::Roll;
pub use types::{Kind, RISE};

#[cfg(test)]
mod tests;
