use std::rc::Rc;

use gpui_kit::{App, Window};

/// The parts shown, at least, when the path folds.
pub const MIN_SHOWN: usize = 3;

pub const DEFAULT_SHOWN: usize = 4;

/// A part's least height: the height of an editor tab beside it.
pub const HEIGHT: f32 = 28.;

/// A link's padding across, and its icon: an editor tab's.
pub const LINK_PAD: f32 = 10.;

pub const ICON: f32 = 14.;

/// How far below, in px, a part starts when it arrives, and how long it takes.
pub const ENTER_Y: f32 = 6.;

pub(super) const ENTER_SECONDS: f32 = 0.2;

pub(super) type Press = Rc<dyn Fn(usize, &mut Window, &mut App)>;
