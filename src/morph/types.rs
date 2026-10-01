use std::rc::Rc;

use gpui_kit::{AnyElement, App, Window};

pub(super) type RenderChild = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;
