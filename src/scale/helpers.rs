use std::cell::Cell;

use gpui_kit::Pixels;

use super::types::{MAX, MIN, STEP};

thread_local! {
    pub(super) static ZOOM: Cell<f32> = const { Cell::new(1.) };
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
