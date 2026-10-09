use gpui_kit::{App, ElementId, SharedString, Window};
use std::rc::Rc;

use crate::update_button::{enums::UpdateState, structs::UpdateButton};

impl UpdateButton {
    /// Starts as a download at 0.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), state: UpdateState::Downloading(0.), label: SharedString::default(), on_click: None }
    }

    /// The update is downloading, `fraction` of it (0 to 1).
    pub fn downloading(mut self, fraction: f32, label: impl Into<SharedString>) -> Self {
        self.state = UpdateState::downloading(fraction);
        self.label = label.into();
        self
    }

    /// The update is ready to install.
    pub fn ready(mut self, label: impl Into<SharedString>) -> Self {
        self.state = UpdateState::Ready;
        self.label = label.into();
        self
    }

    /// The app is restarting into the update; it cannot be pressed.
    pub fn restarting(mut self, label: impl Into<SharedString>) -> Self {
        self.state = UpdateState::Restarting;
        self.label = label.into();
        self
    }

    /// Fires on a press, only while ready.
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}
