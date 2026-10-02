use std::rc::Rc;

use gpui_kit::{App, Window};

use crate::task_edit::Change;

pub const WIDTH: f32 = 240.;

/// What a key did to an open picker.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// The picker is still open; it may have changed.
    Open,
    /// Escape: close it.
    Close,
    /// Enter on a candidate. Close the picker unless `stays_open` (labels), and apply `change`.
    Chosen { change: Change, stays_open: bool },
}

/// What a press on the `n`th shown row does.
pub type Pick = Rc<dyn Fn(usize, &mut Window, &mut App)>;
