use std::rc::Rc;

use gpui_kit::{App, Window};

use crate::{
    motion::{Spring},
    };

/// The shell's spring: `duration 0.8, bounce 0.2`.
pub const SHELL: Spring = Spring { stiffness: 61.685, damping: 12.566, mass: 1. };

/// The corner.
pub const RADIUS: f32 = 32.;

/// The web's compact pill.
pub const PILL: (f32, f32) = (126., 37.);

/// The height of the chip in the title bar: atelier's control size.
pub const HEIGHT: f32 = 28.;

pub(super) type Press = Rc<dyn Fn(&mut Window, &mut App)>;
