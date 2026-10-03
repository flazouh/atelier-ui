use std::{rc::Rc, time::Duration};

use gpui_kit::{App, SharedString, Window};

use super::structs::MenuItem;

/// The text of a row and its line: atelier's body text.
pub const TEXT: f32 = 14.;

pub const LINE: f32 = 20.;

/// The width of the slot that holds a check, a dot or an icon.
pub const SLOT: f32 = 16.;

/// The size of a lead's mark or monogram, in the slot of an icon.
pub(super) const LEAD: f32 = 14.;

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

/// What a menu built from a [`Branch`] tree does with the leaf the reader chose, by its id.
pub type Pick = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// What stands before a row's words to say what it is: an agent's or a lab's mark, or, for one that has no mark, the
/// first letter of its words in a round.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lead {
    Mark(crate::model_badge::BrandMark),
    Monogram,
}

impl Lead {
    /// The mark when there is one, the monogram when there is not: a row never goes without.
    pub fn of(mark: Option<crate::model_badge::BrandMark>) -> Self {
        mark.map_or(Self::Monogram, Self::Mark)
    }
}

/// A choice in a tree of choices. A leaf is chosen; a branch opens a menu of its own beside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch {
    /// What `pick` hears when a leaf is chosen. A branch's id is never heard.
    pub id: SharedString,
    pub label: SharedString,
    pub lead: Option<Lead>,
    pub branches: Vec<Branch>,
}

impl Branch {
    pub fn leaf(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self { id: id.into(), label: label.into(), lead: None, branches: Vec::new() }
    }

    pub fn with(id: impl Into<SharedString>, label: impl Into<SharedString>, branches: Vec<Branch>) -> Self {
        Self { id: id.into(), label: label.into(), lead: None, branches }
    }

    /// The same, with `lead` before its words.
    pub fn led(mut self, lead: Lead) -> Self {
        self.lead = Some(lead);
        self
    }
}

pub(super) type Choose = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Draws the mark at the start of a row.
pub(super) type LeadElement = Rc<dyn Fn(&App) -> gpui_kit::AnyElement>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Default,
    Destructive,
}

/// A row that holds a state. A check and one choice of several both show a check at the row's start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Check(bool),
    Radio(bool),
    /// The chosen row of several: a check at the row's end, as a select shows it.
    Selected(bool),
    /// A switch at the row's end. The row is the control: a press anywhere on it changes the state, and the switch only shows it.
    Switch(bool),
}

/// A heading's height: 6px above, a 16px line and 4px below.
pub(super) const LABEL: f32 = 26.;

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
