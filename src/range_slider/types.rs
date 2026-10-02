use std::rc::Rc;

use gpui_kit::{App, Window};

/// The track: `h-10`, `rounded-lg`. The fill is inset `2px` at each end, the handle starts `8px` in and its
/// travel is `width - 20`, and the ticks run `10px` in from each end.
pub(super) const TRACK_HEIGHT: f32 = 40.;

pub(super) const FILL_INSET: f32 = 2.;

pub(super) const HANDLE_START: f32 = 8.;

pub(super) const TRAVEL_LOSS: f32 = 20.;

pub(super) const TICK_INSET: f32 = 10.;

pub(super) const HANDLE_WIDTH: f32 = 4.;

pub(super) const HANDLE_HEIGHT: f32 = 24.;

pub(super) const TICK: f32 = 4.;

/// A grabbed handle stretches to this.
pub(super) const GRAB_SCALE: f32 = 1.35;

/// `bg-foreground/15`, `bg-foreground/25`, `ring-foreground/30`.
pub(super) const FILL_ALPHA: f32 = 0.15;

pub(super) const TICK_ALPHA: f32 = 0.25;

pub(super) const FOCUS_ALPHA: f32 = 0.3;

pub(super) const DISABLED: f32 = 0.5;

/// The compact slider, for a setting in a row: a 4px rail with the ink filled up to a 12px knob, in a 20px band
/// that takes the presses. No dots.
pub(super) const COMPACT_HEIGHT: f32 = 20.;

pub(super) const RAIL: f32 = 4.;

pub(super) const KNOB: f32 = 12.;

/// The rail's unfilled part: the ink at this.
pub(super) const RAIL_ALPHA: f32 = 0.15;

/// More steps than this draw no dots.
pub(super) const MOST_TICKS: usize = 50;

/// The width assumed before the first layout.
pub(super) const FIRST_WIDTH: f32 = 292.;

pub type ChangeHandler = Rc<dyn Fn(f32, &mut Window, &mut App)>;
