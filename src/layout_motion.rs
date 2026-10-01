//! A child that keeps its place on screen when the layout moves it, and glides to its new place on a spring:
//! Motion's `layout="position"`, done the way FLIP does it. After each frame the child's layout origin is
//! remembered. When the next layout puts it somewhere else, it is drawn, in that same frame, where it stood,
//! that is at the new place plus `old - new`, and the offset runs to zero on the spring. Under Reduce Motion
//! there is no offset: the child is at its new place at once.
//!
//! The offset is applied with the window's element offset during prepaint, so what the child holds, its
//! bounds, its hit boxes and what it paints, all move together. Only position is animated, not size.
//!
//! Do not put one under a scrolling container: a scroll moves the layout origin, and the child would follow
//! the scroll on a spring instead of with it.

mod helpers;
mod structs;
mod types;

pub use helpers::{moved, shifted, start_offset};
pub use structs::Shifted;

#[cfg(test)]
mod tests;
