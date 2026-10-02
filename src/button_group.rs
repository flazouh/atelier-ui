//! ButtonGroup: buttons joined into one control, as gpui-component's `ButtonGroup`
//! (`vendor/gpui-component/src/button/button_group.rs`) joins them, reskinned for atelier-ui. The segments
//! touch, the group keeps the button's corner on its outer corners only, and every segment takes the
//! group's variant and size, so they share one height, one fill and threadmail's press, each on the
//! segment pressed.
//!
//! Borderless: no line parts the segments. A 1px seam does, the surface under the group showing
//! through, so it reads as one button with parts on the page and on a card, in every theme.
//!
//! Each segment keeps its own state: one may be disabled while the others stay live, as the merge
//! button's action greys while its arrow still opens the menu. Each segment is a stop in the Tab order.

mod helpers;
mod structs;
mod types;

pub use helpers::segment_corners;
pub use structs::ButtonGroup;
pub use types::SEAM;

#[cfg(test)]
use gpui_kit::{Axis, Corners, IntoElement, Window, div};
#[cfg(test)]
use crate::scale::px;
#[cfg(test)]
use crate::button::Button;

#[cfg(test)]
mod tests;
