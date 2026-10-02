use std::rc::Rc;

use gpui_kit::{App, ClickEvent, Window};

/// The small size is a `Sm` button's: height, padding across, gap, words and line.
pub const HEIGHT: f32 = 28.;

pub const PAD_X: f32 = 10.;

pub const GAP: f32 = 6.;

pub const TEXT: f32 = 11.;

pub const LINE: f32 = 16.;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwapSize {
    /// A `Sm` button's size.
    #[default]
    Small,
    /// An `Md` button's size.
    Medium,
}

impl SwapSize {
    /// Height, padding across, gap, words and line.
    pub fn metrics(self) -> (f32, f32, f32, f32, f32) {
        match self {
            SwapSize::Small => (HEIGHT, PAD_X, GAP, TEXT, LINE),
            SwapSize::Medium => (32., 12., 8., 12., 16.),
        }
    }
}

/// A press pulls each side in by this share of the width and the height.
pub const PRESS_INSET: f32 = 0.015;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwapVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
}

pub(super) type Click = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
