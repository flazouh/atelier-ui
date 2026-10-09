//! UpdateButton: the title-bar button of the app's background updates. One button in three states.
//!
//! - Downloading: a ring that fills clockwise from 12 o'clock inside a button with a wash of the ink, and a label such
//!   as "Updating 55%". The ring eases toward each new fraction.
//! - Ready: the page inverted like the primary button, a download icon and a label such as "Update to v0.1.9". Only
//!   this state can be pressed (pointer, Enter or Space).
//! - Restarting: the same button as Downloading, but the ring is a quarter arc that turns; under Reduce Motion it
//!   stands still.
//!
//! Its metrics are its own (see `consts.rs`), not the `Button`'s: they come from the mockup Alex chose.
mod consts;
mod enums;
mod helpers;
mod impls;
mod structs;
pub use consts::{GAP, HEIGHT, ICON, PAD_LEFT, PAD_RIGHT, RING, RING_STROKE, TEXT};
pub use enums::UpdateState;
pub use structs::UpdateButton;
#[cfg(test)]
mod tests;
