use crate::{
    icon::{IconName},
    theme::{Theme},
};
use super::structs::BloomItem;
use super::types::{
    CELL_H, COLUMNS, HEADER, IRIS_SIDE, IRIS_TOP, ITEM_DELAY, ITEM_STEP, PANEL_W, TRIGGER_H,
    TRIGGER_W,
};

/// The web's six choices.
pub fn default_items() -> Vec<BloomItem> {
    vec![
        BloomItem::new("Doc", IconName::Description),
        BloomItem::new("Board", IconName::Dashboard),
        BloomItem::new("Table", IconName::TableChart),
        BloomItem::new("Folder", IconName::Folder),
        BloomItem::new("Reminder", IconName::Notifications),
        BloomItem::new("Link", IconName::Link),
    ]
}

/// How many rows the grid has for `count` choices.
pub fn rows(count: usize) -> usize {
    count.div_ceil(COLUMNS).max(1)
}

/// The panel's size: 420 wide, and the header and the rows tall, with the border.
pub fn panel_size(count: usize) -> (f32, f32) {
    (PANEL_W, HEADER + grid_height(count) + 2.)
}

/// The grid's height: the rows, and the 1px line under each row but the last.
pub fn grid_height(count: usize) -> f32 {
    CELL_H * rows(count) as f32 + (rows(count) - 1) as f32
}

/// How far choice `index` is from the grid's centre, in cells: the four corners of a 3x2 grid are equally far.
pub fn distance(index: usize, count: usize) -> f32 {
    let (col, row) = ((index % COLUMNS) as f32, (index / COLUMNS) as f32);
    let (mid_col, mid_row) = ((COLUMNS - 1) as f32 / 2., (rows(count) - 1) as f32 / 2.);
    ((col - mid_col).powi(2) + (row - mid_row).powi(2)).sqrt()
}

/// The seconds before choice `index` starts to arrive.
pub fn delay(index: usize, count: usize) -> f32 {
    ITEM_DELAY + distance(index, count) * ITEM_STEP
}

/// The box `morph` of the way from the button to the panel, as its width and height; both share one centre.
pub fn box_size(morph: f32, count: usize) -> (f32, f32) {
    let (pw, ph) = panel_size(count);
    (TRIGGER_W + (pw - TRIGGER_W) * morph, TRIGGER_H + (ph - TRIGGER_H) * morph)
}

/// What the iris still cuts off the grid at `progress` (0 to 1): the top and bottom, and the two sides, as shares.
pub fn iris_cut(progress: f32) -> (f32, f32) {
    (IRIS_TOP * (1. - progress), IRIS_SIDE * (1. - progress))
}

pub(super) fn hairline(theme: &Theme) -> gpui_kit::Hsla {
    theme.foreground.opacity(0.08)
}
