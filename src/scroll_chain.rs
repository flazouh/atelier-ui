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
//!
//! A box that is not clipped keeps no listener at all, and no handle's stale offset speaks for it: a body clipped to a
//! few rows leaves the wheel to the panel, whatever it did when it was open.
//!
//! A box that follows output (a log, a diff being written) pins its end only while the reader has not scrolled away
//! from it ([`follows`]), as the conversation does ([`FOLLOW_REACH`] there too).

use gpui_kit::{App, Pixels, Point, ScrollHandle, ScrollWheelEvent, Window};

/// Whether a box with scroll `offset` (zero at the start, negative as it moves on) and room to move `max` used a
/// wheel `delta`, which has already been added to `offset`.
pub(crate) fn used_the_wheel(offset: Point<Pixels>, max: Point<Pixels>, delta: Point<Pixels>) -> bool {
    let inside = |offset: Pixels, max: Pixels| offset <= Pixels::ZERO && offset >= -max;
    let moved_y = delta.y != Pixels::ZERO && max.y > Pixels::ZERO && inside(offset.y, max.y);
    let moved_x = delta.x != Pixels::ZERO && max.x > Pixels::ZERO && inside(offset.x, max.x);
    moved_y || moved_x
}

/// How near its end a box must be, in px, for it to keep following output. The conversation's reach, so the two agree.
pub(crate) const FOLLOW_REACH: f32 = 56.;

/// Whether a box that pins its end keeps doing so, given how far (`from_end`, px) its view is from the end after the
/// last layout. While it follows, it is at the end each frame, so any distance past the reach is the reader scrolling
/// away: it lets go, and takes hold again only when they come back to the end.
pub(crate) fn follows(following: bool, from_end: f32) -> bool {
    if following {
        from_end <= FOLLOW_REACH
    } else {
        from_end <= 1.
    }
}

/// How far, in px, the view of a box tracked by `handle` is from its end.
pub(crate) fn from_end(handle: &ScrollHandle) -> f32 {
    f32::from(handle.max_offset().y + handle.offset().y)
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
