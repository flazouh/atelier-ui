use std::rc::Rc;

use gpui_kit::{App, Window};

/// Every sidebar row is this tall, so the list is virtual.
pub const ROW_HEIGHT: f32 = 32.;

/// The box a mark is drawn in.
pub const MARK_BOX: f32 = 16.;

pub(super) type Handler = Rc<dyn Fn(&mut Window, &mut App)>;
