use std::rc::Rc;

use gpui_kit::AnyElement;

use crate::motion::Spring;

/// The rise, for icons and marks.
pub const RISE: Spring = Spring {
    stiffness: 210.,
    damping: 24.,
    mass: 0.85,
};

/// What a roll is made for: a mark (an icon) or words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Icon,
    Words,
    /// Action Swap's roll (`components/motion/action-swap.tsx`): 90% below on `Spring::SWAP`, clear to full over 250ms,
    /// and out through the top to clear in 140ms.
    Swap,
    /// Digit Swap's glyph (`components/motion/digit-swap.tsx`): 45% below, clear to full, over 180ms, and out through the top.
    Digit,
}

impl Kind {
    /// How far below, as a share of the height, the new content starts.
    pub fn start(self) -> f32 {
        match self {
            Kind::Icon => 0.8,
            Kind::Words => 0.85,
            Kind::Swap => 0.9,
            Kind::Digit => 0.45,
        }
    }

    /// The spring the new content rises on.
    pub fn spring(self) -> Spring {
        match self {
            Kind::Swap => Spring::SWAP,
            // A critical spring settles in about the 180ms of the web's tween.
            Kind::Digit => Spring::critical(32.),
            _ => RISE,
        }
    }

    /// The opacity the old content leaves at.
    pub fn exit_opacity(self) -> f32 {
        match self {
            Kind::Swap | Kind::Digit => 0.,
            _ => 0.5,
        }
    }

    /// The opacity the new content starts at, and how long it takes to reach 1.
    pub fn enter_opacity(self) -> (f32, f32) {
        match self {
            Kind::Icon => (0.72, 0.28),
            Kind::Words => (0.76, 0.3),
            Kind::Swap => (0., 0.25),
            Kind::Digit => (0., 0.18),
        }
    }

    /// How long the old content takes to leave.
    pub fn exit_seconds(self) -> f32 {
        match self {
            Kind::Icon => 0.22,
            Kind::Words => 0.2,
            Kind::Swap => 0.14,
            Kind::Digit => 0.18,
        }
    }
}

pub(super) type Build<K> = Rc<dyn Fn(&K) -> AnyElement>;
