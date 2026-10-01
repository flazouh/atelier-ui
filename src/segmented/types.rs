use std::rc::Rc;

use gpui_kit::{App, Window};

/// The gap between segments: the old rows of buttons had 2px.
pub const GAP: f32 = 2.;

/// A segment's height and side padding: a `Sm` button's.
pub const SEGMENT_HEIGHT: f32 = 28.;

pub const SEGMENT_PAD: f32 = 10.;

/// The text: a `Sm` button's, and its line.
pub const TEXT: f32 = 11.;

pub const LINE: f32 = 16.;

/// The gap between the track and a toggle key's cap.
pub const CAP_GAP: f32 = 8.;

/// How long fill and text take to change (`duration-150`).
pub(super) const CHANGE: f32 = 0.15;

/// How far a pressed segment shrinks (`active:scale-95`).
pub(super) const PRESS_SCALE: f32 = 0.95;

/// Tailwind's default easing, `cubic-bezier(0.4, 0, 0.2, 1)`.
pub(super) const EASE: [f32; 4] = [0.4, 0., 0.2, 1.];

pub(super) type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
