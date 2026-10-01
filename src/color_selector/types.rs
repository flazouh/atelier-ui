use std::rc::Rc;

use gpui_kit::{App, SharedString, Window};

/// The swatch's disc, the dot in it, and the space between swatches, in the size of a Settings row's controls.
pub(super) const DISC: f32 = 28.;

pub(super) const DOT: f32 = 14.;

pub(super) const GAP: f32 = 8.;

/// The list's padding: `p-1`.
pub(super) const PAD: f32 = 4.;

/// The ring sits 2px outside the disc, and is `border-2`.
pub(super) const RING_OUT: f32 = 2.;

pub(super) const RING_WIDTH: f32 = 2.;

/// The keyboard outline: `outline-2 outline-offset-4`, outside the ring.
pub(super) const OUTLINE_OFFSET: f32 = 2.;

pub(super) const OUTLINE_WIDTH: f32 = 2.;

/// A press sinks the swatch to this scale (`whileTap`).
pub(super) const PRESS_SCALE: f32 = 0.94;

/// `color-mix(in srgb, <colour> 65%, transparent)`.
pub(super) const RING_ALPHA: f32 = 0.65;

/// The disc's fill is `bg-muted/60`; its edge `border-foreground/10`; the dot's edge `border-black/5`.
pub(super) const DISC_FILL: f32 = 0.6;

pub(super) const DISC_EDGE: f32 = 0.1;

pub(super) const DOT_EDGE: f32 = 0.05;

/// A swatch that cannot be chosen.
pub(super) const UNAVAILABLE: f32 = 0.4;

pub type ChangeHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
