use std::rc::Rc;

use gpui_kit::{App, SharedString, Window};

pub(super) const CONTEXT: &str = "ChangedFileTree";

/// How far each level steps in.
pub(super) const INDENT: f32 = 12.;

pub(super) type OpenHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TreeKey {
    Up,
    Down,
    Left,
    Right,
    Enter,
}
