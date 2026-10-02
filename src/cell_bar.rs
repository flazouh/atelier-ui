//! CellBar: the one loading and progress bar. A row of small rounded cells that light from the left in step with a number, or,
//! when there is no number, one cell that steps along the row.
//!
//! Anything in atelier that waits on something with a size to it (a download, an upload, a model that loads) draws this, so
//! the waiting looks the same everywhere. It draws only: the owner eases the number, because the owner knows how fast it
//! arrives, and says how long it has been loading when there is no number.
//!
//! - Cells: 4px wide, 6px tall, 2px apart, 1.5px corners by default; [`CellBar::stretch`] shares a row's width among
//!   them instead, for a bar that spans a card.
//! - Lit cells take the bar's color at full strength; the head cell is lit in proportion to the number, so the fill moves
//!   smoothly between two whole cells.
//! - Dark cells are a wash of the text color (`foreground` at 12%), not of the bar's own color, so they show on a light page
//!   and on a dark one alike.
//! - Loading: one cell lit, stepping to the next five times a second. Reduce Motion: it rests in the middle.
use gpui_kit::{App, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window, div, prelude::FluentBuilder};

use crate::{scale::px, theme::ActiveTheme};

/// How many cells, and each one's width, height and the space between, in pixels, by default.
pub const CELLS: usize = 14;
pub const CELL_WIDTH: f32 = 4.;
pub const CELL_HEIGHT: f32 = 6.;
pub const CELL_GAP: f32 = 2.;
/// How many cells the loading light passes in a second.
pub const STEPS_PER_SECOND: f32 = 5.;
/// How strong a dark cell is: the text color at this opacity.
pub const DARK_CELL: f32 = 0.12;
/// How fast a number should ease toward its target, per second, for an owner that smooths it with [`pour`].
const POUR_RATE: f32 = 9.;

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

#[derive(IntoElement)]
pub struct CellBar {
    /// How full, 0 to 1, or `None` while there is no number.
    fill: Option<f32>,
    cells: usize,
    cell: (f32, f32),
    stretch: bool,
    color: Option<Hsla>,
    seconds: f32,
    moving: bool,
    name: Option<String>,
}

impl CellBar {
    /// A bar `fill` full (`None` while there is no number to show).
    pub fn new(fill: Option<f32>) -> Self {
        Self { fill, cells: CELLS, cell: (CELL_WIDTH, CELL_HEIGHT), stretch: false, color: None, seconds: 0., moving: true, name: None }
    }

    pub fn cells(mut self, cells: usize) -> Self {
        self.cells = cells.max(1);
        self
    }

    /// Each cell's width and height, in pixels.
    pub fn cell_size(mut self, width: f32, height: f32) -> Self {
        self.cell = (width, height);
        self
    }

    /// Cells share the row's width evenly instead of keeping their own, so the bar spans what holds it.
    pub fn stretch(mut self, stretch: bool) -> Self {
        self.stretch = stretch;
        self
    }

    /// The lit cells' color. Without one, the text color.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// How long it has been loading, for the stepping cell; only read while there is no number.
    pub fn seconds(mut self, seconds: f32) -> Self {
        self.seconds = seconds;
        self
    }

    /// Off under Reduce Motion: the loading cell rests.
    pub fn moving(mut self, moving: bool) -> Self {
        self.moving = moving;
        self
    }

    /// The name a test finds the bar by with `debug_bounds`.
    pub fn debug_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl RenderOnce for CellBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let color = self.color.unwrap_or(theme.foreground);
        let dark = theme.foreground.opacity(DARK_CELL);
        let (width, height) = self.cell;
        let (stretch, cells) = (self.stretch, self.cells);
        let loading = loading_cell(self.seconds, cells, self.moving);
        let fill = self.fill;
        let cell = (0..cells).map(move |i| {
            let on = match fill {
                Some(fill) => lit(i, cells, fill),
                None => (i == loading) as u8 as f32,
            };
            let cell = div().h(px(height)).rounded(px(1.5)).bg(cell_color(color, dark, on));
            if stretch { cell.flex_1() } else { cell.flex_none().w(px(width)) }
        });
        let row = div().flex().items_center().gap(px(CELL_GAP)).when(stretch, |d| d.w_full()).when(!stretch, |d| d.flex_none()).children(cell);
        match self.name {
            Some(name) => row.debug_selector(move || name.clone()),
            None => row,
        }
    }
}

#[cfg(test)]
mod tests;
