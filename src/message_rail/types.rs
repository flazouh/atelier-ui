use std::rc::Rc;

use gpui_kit::{App, Window};

/// The longest a message's label and its answer's start are, as on the web.
pub const LABEL_CHARS: usize = 56;

pub const DESCRIPTION_CHARS: usize = 88;

pub(super) type Handler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
