//! Plays an animated mark from a sprite strip: an SVG that stacks its frames top to bottom.
//!
//! A [`Strip`] is plain data: where the SVG is served, its bytes, how many frames it holds, how long each
//! shows, and whether it loops. beui ships no strips of its own; an agent's strips reach it as data (see
//! [`crate::agent_look::AgentLook`]). Whoever owns the strips also serves them to GPUI under
//! [`Strip::path`], since [`Sprite`] draws with `svg().path()`.
//!
//! The strip plays as CSS `steps(frames, jump-none)`: each frame shows for the same time. GPUI has no
//! element transform, so [`Sprite`] clips a `size x size` box over the strip and moves the strip up by
//! whole frames.
//!
//! It wakes the view once per frame period with a timer ([`Wake`]), not every display frame. Under Reduce
//! Motion, or when it is not playing, it draws the first frame of its rest strip and stands still.
//!
//! Looping strips read a process-wide clock ([`shared_clock`]) instead of when that particular sprite
//! mounted, so every sprite showing the same looping strip lands on the same frame at the same moment and
//! they wake for the same redraw instead of drifting apart. One-shots keep their own start, since each
//! plays its own run once from the moment it is triggered.

mod helpers;
mod structs;

#[cfg(test)]
pub(crate) use helpers::{frame_height, native_size, restarts};
pub use structs::{Sprite, Strip};

#[cfg(test)]
mod tests;
