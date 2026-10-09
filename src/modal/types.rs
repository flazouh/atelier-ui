use std::rc::Rc;

use gpui_kit::{App, Window};

use crate::motion::Spring;

/// The padding round the view, and the border (none: atelier is borderless).
pub const PAD: f32 = 16.;

pub const BORDER: f32 = 0.;

/// The panel's corner: atelier's card radius for a floating surface.
pub const CORNER: f32 = 12.;

/// How far below the panel starts (`enterY` for the centre placement).
pub const ENTER_Y: f32 = 20.;

/// The panel's spring.
pub const PANEL: Spring = Spring::PANEL;

/// How long the scrim takes to come, and a new view.
pub(super) const SCRIM_SECONDS: f32 = 0.2;

pub(super) const VIEW_SECONDS: f32 = 0.24;

/// How far below a new view starts.
pub const VIEW_Y: f32 = 8.;

pub(super) type Close = Rc<dyn Fn(&mut Window, &mut App)>;

/// The room a panel leaves between itself and the window's edge, in design pixels: a tall view scrolls inside what is left.
pub const EDGE: f32 = 24.;
