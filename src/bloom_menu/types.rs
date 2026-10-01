use gpui_kit::SharedString;

/// The button: `h-11 w-36`, `rounded-2xl`.
pub(super) const TRIGGER_W: f32 = 144.;

pub(super) const TRIGGER_H: f32 = 44.;

pub(super) const RADIUS: f32 = 16.;

/// The panel: the header, `px-4 py-3` round a line of words, and rows of cells of `px-3 py-6` round an icon and a label.
pub(super) const PANEL_W: f32 = 420.;

pub(super) const HEADER: f32 = 45.;

pub(super) const CELL_W_PAD: f32 = 12.;

pub(super) const CELL_H: f32 = 96.;

pub(super) const COLUMNS: usize = 3;

/// The words arrive after this long, over this long; the iris after its own delay, over its own time.
pub(super) const WORDS_DELAY: f32 = 0.12;

pub(super) const WORDS_TIME: f32 = 0.2;

pub(super) const IRIS_DELAY: f32 = 0.08;

pub(super) const IRIS_TIME: f32 = 0.45;

/// The iris starts as this share of the grid's height and width cut off each side.
pub(super) const IRIS_TOP: f32 = 0.45;

pub(super) const IRIS_SIDE: f32 = 0.34;

/// A choice arrives after this long, and this much longer for each step from the centre.
pub(super) const ITEM_DELAY: f32 = 0.1;

pub(super) const ITEM_STEP: f32 = 0.07;

pub(super) const ITEM_FROM: f32 = 0.85;

pub(super) const PRESSED: f32 = 0.97;

pub enum BloomEvent {
    /// A choice was made; the menu shuts.
    Select(SharedString),
    /// The menu opened or shut.
    Toggled(bool),
}
