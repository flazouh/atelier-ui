use gpui_kit::{
    App, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder,
};

use super::helpers::{cell_color, lit, loading_cell};
use super::types::{CELL_GAP, CELL_HEIGHT, CELL_WIDTH, CELLS, DARK_CELL};
use crate::{scale::px, theme::ActiveTheme};

#[derive(IntoElement)]
pub struct CellBar {
    /// How full, 0 to 1, or `None` while there is no number.
    pub(super) fill: Option<f32>,
    pub(super) cells: usize,
    pub(super) cell: (f32, f32),
    pub(super) stretch: bool,
    pub(super) color: Option<Hsla>,
    pub(super) seconds: f32,
    pub(super) moving: bool,
    pub(super) name: Option<String>,
}

impl CellBar {
    /// A bar `fill` full (`None` while there is no number to show).
    pub fn new(fill: Option<f32>) -> Self {
        Self {
            fill,
            cells: CELLS,
            cell: (CELL_WIDTH, CELL_HEIGHT),
            stretch: false,
            color: None,
            seconds: 0.,
            moving: true,
            name: None,
        }
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
            let cell = div()
                .h(px(height))
                .rounded(px(1.5))
                .bg(cell_color(color, dark, on));
            if stretch {
                cell.flex_1()
            } else {
                cell.flex_none().w(px(width))
            }
        });
        let row = div()
            .flex()
            .items_center()
            .gap(px(CELL_GAP))
            .when(stretch, |d| d.w_full())
            .when(!stretch, |d| d.flex_none())
            .children(cell);
        match self.name {
            Some(name) => row.debug_selector(move || name.clone()),
            None => row,
        }
    }
}
