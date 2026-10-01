//! The interface's scale, one value, as in Zed (which sets its rem size from the UI font size, so all of its layout
//! follows). Every pixel size in atelier goes through [`px`] here: `px(14.)` is 14 design pixels times the zoom, so
//! text, icons, rows, padding and widths all scale together, and ⌘+ ⌘− ⌘0 change one number.
//!
//! The zoom lives in a thread-local: the interface is drawn on one thread, and a test that zooms cannot change what the
//! tests on other threads measure. Sizes in the window's own pixels (the viewport, a measured bound) are divided by
//! [`zoom`] to give design pixels, and a design size drawn goes back through [`px`].
use std::cell::Cell;

use gpui_kit::Pixels;

/// The least and the most the interface scales to, and the step of ⌘+ and ⌘−.
pub const MIN: f32 = 0.7;
pub const MAX: f32 = 2.0;
pub const STEP: f32 = 0.1;

thread_local! {
    static ZOOM: Cell<f32> = const { Cell::new(1.) };
}

/// The zoom now: 1 is the interface as designed.
pub fn zoom() -> f32 {
    ZOOM.with(Cell::get)
}

/// Sets the zoom, kept between [`MIN`] and [`MAX`] and to a tenth. Returns what it set.
pub fn set_zoom(zoom: f32) -> f32 {
    let kept = ((zoom.clamp(MIN, MAX)) / STEP).round() * STEP;
    ZOOM.with(|z| z.set(kept));
    kept
}

/// `design` pixels at the zoom now. Use this wherever a size is written.
pub fn px(design: f32) -> Pixels {
    gpui_kit::px(design * zoom())
}

/// The design pixels that `actual` window pixels are at the zoom now.
pub fn design(actual: Pixels) -> f32 {
    f32::from(actual) / zoom()
}

#[cfg(test)]
mod tests;
