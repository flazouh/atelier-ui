use gpui_kit::Hsla;

use super::types::{POUR_RATE, STEPS_PER_SECOND};

/// `current` after `dt` seconds of easing toward `target`, for an owner that smooths a number that arrives in jumps.
pub fn pour(current: f32, target: f32, dt: f32) -> f32 {
    current + (target - current) * (1. - (-POUR_RATE * dt).exp())
}

/// How lit cell `i` of `cells` is, 0 to 1, when the bar is `fill` full.
pub fn lit(i: usize, cells: usize, fill: f32) -> f32 {
    let fill = if fill.is_nan() { 0. } else { fill.clamp(0., 1.) };
    (fill * cells as f32 - i as f32).clamp(0., 1.)
}

/// How many cells are lit all the way at `fill`.
pub fn whole_cells(cells: usize, fill: f32) -> usize {
    (0..cells).filter(|&i| lit(i, cells, fill) >= 1.).count()
}

/// The cell the loading light is on `seconds` in: it goes along the row and starts again at the left. With `moving` off it
/// rests in the middle.
pub fn loading_cell(seconds: f32, cells: usize, moving: bool) -> usize {
    if moving { (seconds * STEPS_PER_SECOND) as usize % cells.max(1) } else { cells / 2 }
}

/// The color of a cell that is `lit` (0 to 1): the dark cell's wash while it is dark, and from the first light on `color`,
/// its strength rising from the wash's to `color`'s own. A half-lit head cell is so a paler version of the bar's color.
pub fn cell_color(color: Hsla, dark: Hsla, lit: f32) -> Hsla {
    let lit = lit.clamp(0., 1.);
    if lit <= 0. {
        return dark;
    }
    Hsla { a: dark.a + (color.a - dark.a) * lit, ..color }
}
