//! One focus style for every field, button and row that the app owns: a 2px ring just outside the control, in the
//! ink at the least strength that reaches 3:1 on the surface behind it. It is quiet in a theme with a light page and
//! in one with a dark page alike. It is drawn for keyboard focus on a button and row, and for any focus inside a
//! [`Field`] (a text field shows where the caret is, whoever put it there).
//!
//! The parts ported from beui.dev keep the focus look of the web version (a checkbox, a slider, a swatch).

mod helpers;
mod structs;
mod traits;
mod types;

pub use helpers::{ring_color, ring_shadow, row_ring};
pub use structs::Field;
pub use traits::PressStop;
pub use types::RING_WIDTH;

#[cfg(test)]
use crate::theme::{MARK_CONTRAST, contrast};
#[cfg(test)]
use gpui_kit::FocusHandle;

#[cfg(test)]
mod tests;
