//! Lets a scroller inside a scroller hand the wheel on, as a browser does.
//!
//! GPUI gives a wheel event to every scroller under the pointer, innermost first, and none of them stops it. So a
//! diff or an output box inside the agent panel scrolls, and the panel scrolls with it. Stopping the event
//! whenever the box can scroll fixes that, but then the panel is stuck whenever the pointer is over the box.
//!
//! [`keep_inside`] sits on the box's wrapper, which runs after the box's own scroll listener has moved its offset.
//! If that offset is still inside its range, the box used the wheel and the event stops here. If it ran past an end,
//! the box had nothing more to show in that direction (GPUI clamps it on the next frame), so the event goes on to
//! the panel.

use gpui_kit::{App, Pixels, Point, ScrollHandle, ScrollWheelEvent, Window};

/// Whether a box with scroll `offset` (zero at the start, negative as it moves on) and room to move `max` used a
/// wheel `delta`, which has already been added to `offset`.
pub(crate) fn used_the_wheel(offset: Point<Pixels>, max: Point<Pixels>, delta: Point<Pixels>) -> bool {
    let inside = |offset: Pixels, max: Pixels| offset <= Pixels::ZERO && offset >= -max;
    let moved_y = delta.y != Pixels::ZERO && max.y > Pixels::ZERO && inside(offset.y, max.y);
    let moved_x = delta.x != Pixels::ZERO && max.x > Pixels::ZERO && inside(offset.x, max.x);
    moved_y || moved_x
}

/// The wheel handler for the wrapper of a box tracked by `handle`: it stops the event while the box used it.
pub(crate) fn keep_inside(handle: ScrollHandle) -> impl Fn(&ScrollWheelEvent, &mut Window, &mut App) + 'static {
    move |event, window, cx| {
        let delta = event.delta.pixel_delta(window.line_height());
        if used_the_wheel(handle.offset(), handle.max_offset(), delta) {
            cx.stop_propagation();
        }
    }
}

#[cfg(test)]
mod tests;
