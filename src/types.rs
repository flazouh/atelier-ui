use std::rc::Rc;

use gpui_kit::{App, ClickEvent, Window};

/// A click callback that components store and share across frames.
pub(crate) type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
