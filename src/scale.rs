//! The interface's scale, one value, as in Zed (which sets its rem size from the UI font size, so all of its layout
//! follows). Every pixel size in atelier goes through [`px`] here: `px(14.)` is 14 design pixels times the zoom, so
//! text, icons, rows, padding and widths all scale together, and ⌘+ ⌘− ⌘0 change one number.
//!
//! The zoom lives in a thread-local: the interface is drawn on one thread, and a test that zooms cannot change what the
//! tests on other threads measure. Sizes in the window's own pixels (the viewport, a measured bound) are divided by
//! [`zoom`] to give design pixels, and a design size drawn goes back through [`px`].

mod helpers;
mod types;

pub use helpers::{design, px, set_zoom, zoom};
pub use types::{MAX, MIN, STEP};

#[cfg(test)]
mod tests;
