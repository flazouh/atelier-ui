use std::rc::Rc;

use gpui_kit::{App, Window};

use crate::merge::{Action, Choice};

pub type ActionHandler = Rc<dyn Fn(Action, &mut Window, &mut App)>;

pub type ChoiceHandler = Rc<dyn Fn(Choice, &mut Window, &mut App)>;

/// What a menu row does when picked.
pub(super) type Pick = Rc<dyn Fn(&mut Window, &mut App)>;

/// The gap between the panel and the button.
pub(super) const MENU_GAP: f32 = 4.;
