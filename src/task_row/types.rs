use std::rc::Rc;

use gpui_kit::{App, ClickEvent, Window};

pub const ROW_HEIGHT: f32 = 28.;

/// How many labels a row shows before "+n".
pub const MAX_LABELS: usize = 2;

pub(super) type Handler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
