use std::rc::Rc;

use gpui_kit::{App, Window};

use crate::motion::Curve;

/// The box, its edge and corner: `h-5 w-5`, `border-2`, `rounded-md`.
pub(super) const BOX: f32 = 20.;

pub(super) const EDGE: f32 = 2.;

pub(super) const CORNER: f32 = 6.;

/// The mark: a 12px svg on a 24-unit grid, stroked 3 units wide.
pub(super) const MARK: f32 = 12.;

pub(super) const STROKE: f32 = 3.;

/// The tick `M5 13l4 4L19 7` and the dash `M6 12h12`.
pub(super) const TICK: [(f32, f32); 3] = [(5., 13.), (9., 17.), (19., 7.)];

pub(super) const DASH: [(f32, f32); 2] = [(6., 12.), (18., 12.)];

/// The label sits `gap-3` from the box.
pub(super) const GAP: f32 = 12.;

pub(super) const PRESS_SCALE: f32 = 0.92;

/// The mark comes in and goes out over 160ms; the stroke draws for 300ms (200ms for the dash) after 40ms.
pub(super) const FADE: f32 = 0.16;

pub(super) const DRAW_TICK: f32 = 0.3;

pub(super) const DRAW_DASH: f32 = 0.2;

pub(super) const DRAW_DELAY: f32 = 0.04;

/// `transition-colors duration-200`, Tailwind's default curve.
pub(super) const TINT: Curve = Curve::Ease(0.2, [0.4, 0., 0.2, 1.]);

pub(super) const DISABLED: f32 = 0.6;

/// The focus ring is `--ring`, the strong border: the foreground at 12%.
pub(super) const RING_ALPHA: f32 = 0.12;

pub type ChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

pub(super) type Toggle = Rc<dyn Fn(&mut Window, &mut App)>;

/// What the mark shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    None,
    Tick,
    Dash,
}

impl Mark {
    /// A partial choice wins over checked, as in the web version.
    pub fn of(checked: bool, indeterminate: bool) -> Self {
        if indeterminate {
            Mark::Dash
        } else if checked {
            Mark::Tick
        } else {
            Mark::None
        }
    }

    pub(super) fn points(self) -> &'static [(f32, f32)] {
        match self {
            Mark::None => &[],
            Mark::Tick => &TICK,
            Mark::Dash => &DASH,
        }
    }
}
