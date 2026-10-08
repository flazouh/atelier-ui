//! Button, the same as threadmail's: the Fluid Functionalism shape and press, with mem0's arrow chip.
//!
//! - Shape: 8px corners (`rounded-lg`), and 11px text at Small, the default; Medium has fluidfunctionalism.com's 12px.
//!   Four sizes: Small (the default), Medium for an action that must stand out, Large and Xl.
//! - Press: the fill shrinks by 1px on every side, so the button sinks by one pixel (80ms down, 180ms
//!   back), like Fluid Functionalism's collapsing ring.
//! - Chip: an optional square on the right holding an icon. On hover it turns from dark to light while a
//!   white icon slides up and out and a dark one slides up into place (mem0.ai hero).
//! - Color: the primary is the theme's `primary` (`#F9A825`), with near-black text.

mod helpers;
mod structs;
mod types;

pub use helpers::dot;
pub(crate) use helpers::colors;
pub use structs::Button;
pub use types::{ButtonSize, ButtonVariant, ROUND};

#[cfg(test)]
use helpers::hover_target;
#[cfg(test)]
use helpers::side_pad;

#[cfg(test)]
mod tests;
