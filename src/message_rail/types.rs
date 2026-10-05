use std::rc::Rc;

use gpui_kit::{App, Window};

/// The longest a message's label and its answer's start are, as on the web.
pub const LABEL_CHARS: usize = 56;

pub const DESCRIPTION_CHARS: usize = 88;

pub(super) type Handler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The widest the card beside a tick grows: a long message is cut here, a short one is as wide as it is.
pub(super) const CARD_MAX_WIDTH: f32 = 256.;
