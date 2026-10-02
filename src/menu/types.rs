use std::{rc::Rc, time::Duration};

use gpui_kit::{App, SharedString, Window};

use super::structs::MenuItem;

/// The text of a row and its line: atelier's body text.
pub const TEXT: f32 = 14.;

pub const LINE: f32 = 20.;

/// The width of the slot that holds a check, a dot or an icon.
pub const SLOT: f32 = 16.;

/// How long the panel takes to unfold.
pub(super) const UNFOLD: f32 = 0.3;

/// The clip starts as a square this far each side of the origin.
pub(super) const CLIP_HALF: f32 = 8.;

/// The corner of the clip at the start and at the end.
pub(super) const RADIUS_START: f32 = 10.;

/// How much faster than the unfold the fill comes in. The panel sits over text, and a half-clear fill lets that
/// text show through the rows; so the fill is whole a sixth of the way into the unfold, and only the clip grows.
pub(super) const FILL_RAMP: f32 = 6.;

pub(super) const RADIUS_END: f32 = 12.;

/// The letters typed to jump to a row are forgotten after this long.
pub(super) const TYPED_FOR: Duration = Duration::from_millis(500);

pub type Select = Rc<dyn Fn(&mut Window, &mut App)>;

pub(super) type Choose = Rc<dyn Fn(usize, &mut Window, &mut App)>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Default,
    Destructive,
}

/// A row that holds a state: a check for a switch, a dot for one choice of several.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Check(bool),
    Radio(bool),
}

pub enum Entry {
    Item(MenuItem),
    /// A small heading over the rows that follow.
    Label(SharedString),
    Separator,
}

impl From<MenuItem> for Entry {
    fn from(item: MenuItem) -> Self {
        Entry::Item(item)
    }
}

/// Where the panel unfolds from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Origin {
    /// A point in the panel, from its top left corner.
    At(f32, f32),
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Origin {
    /// The point in a panel of `size`, kept 12px from the edges.
    pub fn point(self, size: (f32, f32)) -> (f32, f32) {
        let (x, y) = match self {
            Origin::At(x, y) => (x, y),
            Origin::TopLeft => (0., 0.),
            Origin::TopRight => (size.0, 0.),
            Origin::BottomLeft => (0., size.1),
            Origin::BottomRight => size,
        };
        (x.clamp(12., (size.0 - 12.).max(12.)), y.clamp(12., (size.1 - 12.).max(12.)))
    }
}

/// The panel's border, on each side.
pub const BORDER: f32 = 1.;
