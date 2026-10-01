use std::rc::Rc;

use gpui_kit::{App, Window};

use crate::{
    motion::{Spring},
    };

/// The indicator's spring: `{ stiffness: 245, damping: 36, mass: 1.2 }`.
pub const GLIDE: Spring = Spring { stiffness: 245., damping: 36., mass: 1.2 };

/// An underline tab's least height (`min-h-[44px]`).
pub const UNDERLINE_HEIGHT: f32 = 44.;

/// An editor tab's height: atelier's control size.
pub const EDITOR_HEIGHT: f32 = 28.;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsVariant {
    #[default]
    Pill,
    Underline,
    Segment,
    /// The editor strip's designs, in atelier's look: a 28px tab and no rule. The open tab's marker glides.
    /// A: a `card_strong` chip behind the open tab.
    Chip,
    /// B: the chip and a 2px line at its foot.
    ChipLine,
    /// C: a text tab with a 4px dot under it.
    Dot,
    /// D: a 2px accent tick at the tab's left.
    Tick,
}

impl TabsVariant {
    /// One of the editor strip's designs.
    pub fn is_editor(self) -> bool {
        matches!(self, TabsVariant::Chip | TabsVariant::ChipLine | TabsVariant::Dot | TabsVariant::Tick)
    }
    /// The list's padding.
    pub fn pad(self) -> f32 {
        match self {
            TabsVariant::Pill => 4.,
            TabsVariant::Segment => 2.,
            TabsVariant::Underline | TabsVariant::Chip | TabsVariant::ChipLine | TabsVariant::Dot | TabsVariant::Tick => 0.,
        }
    }

    /// The gap between tabs.
    pub fn gap(self) -> f32 {
        match self {
            TabsVariant::Segment => 0.,
            _ => 4.,
        }
    }

    /// A tab's padding across and down.
    pub fn tab_pad(self) -> (f32, f32) {
        match self {
            TabsVariant::Underline => (12., 0.),
            TabsVariant::Chip | TabsVariant::ChipLine | TabsVariant::Dot | TabsVariant::Tick => (10., 0.),
            _ => (14., 6.),
        }
    }
}

pub(super) type Select = Rc<dyn Fn(usize, &mut Window, &mut App)>;
