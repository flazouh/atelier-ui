use std::rc::Rc;

use gpui_kit::{App, ElementId, IntoElement, SharedString, Window};

use crate::update_button::enums::UpdateState;

/// The handler of a press, for the app to start the restart.
pub(in crate::update_button) type Pressed = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Clone, IntoElement)]
pub struct UpdateButton {
    pub(in crate::update_button) id: ElementId,
    pub(in crate::update_button) state: UpdateState,
    pub(in crate::update_button) label: SharedString,
    pub(in crate::update_button) on_click: Option<Pressed>,
}
