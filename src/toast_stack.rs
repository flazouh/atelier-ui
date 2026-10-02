//! beui's AnimatedToastStack (`components/motion/animated-toast-stack.tsx`): toasts that stack in a corner of the
//! window. A toast is a status icon, a title, an optional description and action, and a close button. It comes in
//! from 22px below its place, fading in on `{ stiffness: 420, damping: 34, mass: 0.75 }`, and leaves by fading out
//! and sliding 32px to the right in 180ms; the toasts round it glide to their new places on the same spring
//! ([`crate::layout_motion::shifted`]). A toast that changes (loading to success) swaps its icon and words in
//! 280ms: the old ones fade up and out, the new ones fade in from below. A toast that is dragged sideways follows
//! the pointer at 0.18 of the distance and leaves when it is dragged past 72px, or thrown faster than 520px/s;
//! else it springs back. A toast goes by itself after its duration, unless that is zero.
//!
//! What gpui cannot draw is left out: the blur on entry and exit, the 0.96 scale, and the backdrop blur.
//! The stack is a layer over its parent: put it as the last child of a `relative` box that fills the window.

mod helpers;
mod structs;
mod types;

pub use helpers::{disc_for, drawn, elastic, lets_go, stack_width};
pub use structs::{Toast, ToastPatch, ToastStack};
pub use types::{DEFAULT_DURATION, ToastEvent, ToastPosition, ToastStatus};

#[cfg(test)]
use types::{EDGE_BOTTOM, EDGE_TOP, EDGE_X, GAP};

#[cfg(test)]
use std::time::Duration;
#[cfg(test)]
use gpui_kit::{Context, IntoElement, Pixels, SharedString, Window, div};
#[cfg(test)]
use crate::scale::px;

#[cfg(test)]
mod tests;
