use gpui_kit::{ElementId, IntoElement, Pixels, Point, point};

use super::structs::Shifted;
use super::types::LEAST;
use crate::motion::Spring;

/// `child`, gliding to wherever the layout moves it. `id` names it across frames, and must be the same each
/// frame for the same child (a chip's value, a toast's number).
pub fn shifted(id: impl Into<ElementId>, child: impl IntoElement) -> Shifted {
    Shifted {
        id: id.into(),
        child: child.into_any_element(),
        spring: Spring::LAYOUT,
    }
}

/// The offset that draws a child where it stood: from the layout origin it had, and the offset it wore, to the
/// origin it has now.
pub fn start_offset(was: Point<Pixels>, wore: Point<Pixels>, now: Point<Pixels>) -> Point<Pixels> {
    point(was.x + wore.x - now.x, was.y + wore.y - now.y)
}

/// Whether the child moved.
pub fn moved(was: Point<Pixels>, now: Point<Pixels>) -> bool {
    (f32::from(was.x) - f32::from(now.x)).abs() > LEAST
        || (f32::from(was.y) - f32::from(now.y)).abs() > LEAST
}
