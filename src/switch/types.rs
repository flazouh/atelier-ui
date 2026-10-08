use std::rc::Rc;

use gpui_kit::{App, Window};

use crate::motion::Spring;

pub const WIDTH: f32 = 32.;

pub const HEIGHT: f32 = 20.;

pub const PAD: f32 = 3.;

pub const THUMB: f32 = 14.;

/// How far the thumb travels.
pub const TRAVEL: f32 = WIDTH - 2. * PAD - THUMB;

/// The thumb's spring: heavy, so the travel is weighty; one overshoot of 4%.
pub const THUMB_SPRING: Spring = Spring {
    stiffness: 800.,
    damping: 80.,
    mass: 4.,
};

/// The thumb's scale and stretch (px toward the side it goes from) while the pointer is down.
pub const SQUEEZE: f32 = 0.9;

pub const STRETCH: f32 = 3.;

/// How long the track takes to change colour.
pub(super) const FILL_SECONDS: f32 = 0.2;

/// A disabled switch's shake, in px at even steps, after a delay.
pub const SHAKE: [f32; 5] = [0., -2., 2., -1., 0.];

pub(super) const SHAKE_DELAY: f32 = 0.2;

pub(super) const SHAKE_SECONDS: f32 = 0.6;

pub(super) const EASE_IN_OUT: [f32; 4] = [0.4, 0., 0.2, 1.];

pub(super) type Change = Rc<dyn Fn(bool, &mut Window, &mut App)>;
