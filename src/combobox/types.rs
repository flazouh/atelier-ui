use std::rc::Rc;

use gpui_kit::{App, SharedString, Window};

use crate::{motion::Spring, theme::{mix, radius}};
use super::structs::ComboRow;

/// The list's padding and its tallest.
pub const PAD: f32 = 6.;

pub const MAX_HEIGHT: f32 = 256.;

/// A row's padding across and down, the gap between its parts, its text and its line.
pub const ROW_PAD_X: f32 = 8.;

pub const ROW_PAD_Y: f32 = 8.;

pub const ROW_GAP: f32 = 8.;

pub const TEXT: f32 = 14.;

pub const LINE: f32 = 20.;

/// The gap between rows of a list.
pub const ROW_GAP_BETWEEN: f32 = 2.;

/// The height of a row.
pub const ROW_HEIGHT: f32 = LINE + 2. * ROW_PAD_Y;

/// How long the check takes to appear, and the size it grows from.
pub(super) const CHECK_SECONDS: f32 = 0.15;

pub(super) const CHECK_FROM: f32 = 0.82;

/// The look of the rows: the Combobox's, or the Command Palette's (`components/motion/command-palette.tsx`), whose rows
/// are `rounded-md` with 12px between the parts, whose pill is `bg-muted/60` and follows the keys on
/// `{ stiffness: 480, damping: 38 }`, and whose list has 8px of padding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComboStyle {
    #[default]
    Combobox,
    Palette,
    /// The folder picker's rows before this part: 32 tall, 10px across, `rounded MD`, the active row on `card_strong`.
    Folder,
    /// The task pickers' rows before this part: 28 tall, 8px across, `rounded MD`.
    Task,
}

impl ComboStyle {
    pub(super) fn gap(self) -> f32 {
        match self {
            ComboStyle::Combobox | ComboStyle::Folder | ComboStyle::Task => ROW_GAP,
            ComboStyle::Palette => 12.,
        }
    }
    /// A row's padding across and down.
    pub fn row_x(self) -> f32 {
        match self {
            ComboStyle::Folder | ComboStyle::Task => 10.,
            _ => ROW_PAD_X,
        }
    }
    pub fn row_y(self) -> f32 {
        match self {
            ComboStyle::Folder => 6.,
            ComboStyle::Task => 4.,
            _ => ROW_PAD_Y,
        }
    }
    /// The corner of a row and of the pill that glides to it.
    pub fn radius(self) -> gpui_kit::Pixels {
        match self {
            ComboStyle::Combobox => radius::lg(),
            _ => radius::md(),
        }
    }
    /// The pill's fill.
    pub fn fill(self, theme: &crate::theme::Theme) -> gpui_kit::Hsla {
        match self {
            ComboStyle::Combobox => mix(theme.card, theme.foreground, 0.06),
            // design preview: remove after Alex picks (the elevation)
            _ => crate::design_preview::row_tone(theme, crate::design_preview::panel_fill(theme, crate::design_preview::elevation(), theme.popover)),
        }
    }

    pub(super) fn pad(self) -> f32 {
        match self {
            ComboStyle::Combobox | ComboStyle::Folder | ComboStyle::Task => PAD,
            ComboStyle::Palette => 8.,
        }
    }

    pub(super) fn spring(self) -> Spring {
        match self {
            ComboStyle::Combobox | ComboStyle::Folder | ComboStyle::Task => Spring::LAYOUT,
            ComboStyle::Palette => PALETTE_SPRING,
        }
    }
}

/// The pill of the command palette follows rapid arrow keys, so it is tighter than the layout spring.
pub const PALETTE_SPRING: Spring = Spring { stiffness: 480., damping: 38., mass: 1. };

pub(super) type Pick = Rc<dyn Fn(usize, &mut Window, &mut App)>;

pub enum ComboEntry {
    Row(ComboRow),
    /// A heading over the rows that follow.
    Group(SharedString),
    Separator,
}

impl From<ComboRow> for ComboEntry {
    fn from(row: ComboRow) -> Self {
        ComboEntry::Row(row)
    }
}
