use std::rc::Rc;

use gpui_kit::{App, Window};

/// The surface's corner, and the item grid it measures from.
pub(super) const CORNER: f32 = 12.;

pub(super) const ITEM_HEIGHT: f32 = 28.;

/// The gap between rows, in every list.
pub const ROW_GAP: f32 = 2.;

/// A group's heading row.
pub(super) const HEADING_HEIGHT: f32 = 26.;

pub(super) const PANEL_PAD: f32 = 4.;

/// Options come in 80ms after the open, 35ms apart (`delayChildren` and `staggerChildren`).
pub(super) const ITEM_DELAY: f32 = 0.08;

pub(super) const ITEM_STEP: f32 = 0.035;

/// Motion's default tween for an option's opacity.
pub(super) const ITEM_FADE: f32 = 0.3;

/// An option starts 6px up and settles on a spring of stiffness 500 and damping 25 (Motion's default for `y`).
pub(super) const ITEM_RISE: f32 = 6.;

pub(super) const RISE_STIFFNESS: f32 = 500.;

pub(super) const RISE_DAMPING: f32 = 25.;

/// Where an option's content starts: the list's padding and the option's own.
pub const OPTION_INSET: f32 = PANEL_PAD + 10.;

pub type SelectHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// How long a pause ends the letters typed so far.
pub(super) const TYPE_AHEAD: std::time::Duration = std::time::Duration::from_millis(700);
