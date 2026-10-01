//! beui's BloomMenu (`components/motion/bloom-menu.tsx`): a "Create +" button that opens into a menu by growing, from its
//! own middle outward, into a 420px panel: a header ("Create" and a close cross) over a grid of three columns of
//! choices. It is the same box the whole way, so the button and the panel are one shape at two sizes, both centred on the
//! button, and it closes the same way back.
//!
//! Motion: the box grows on `{ stiffness: 300, damping: 32, mass: 0.9 }`, a folder opening with a touch of overshoot. The
//! panel's words come in after 120ms, over 200ms. The grid opens as an iris: a small box at its centre (45% of the height and
//! 34% of the width off each edge) that grows to the whole grid in 450ms after 80ms. Each choice comes up on
//! `{ stiffness: 440, damping: 34 }` after `100ms + 70ms` for each step of its distance from the grid's centre, so the four
//! corners arrive together. The button gives to 0.97 while it is pressed. Under Reduce Motion it all jumps.
//!
//! What gpui cannot draw is left out: the blur on the choices as they arrive, and the scale of their labels (their icons
//! do scale).

mod helpers;
mod structs;
mod types;

pub use helpers::{
    box_size, default_items, delay, distance, grid_height, iris_cut, panel_size, rows,
};
pub use structs::{BloomItem, BloomMenu};
pub use types::BloomEvent;

#[cfg(test)]
mod tests;
