use gpui_kit::FocusHandle;

use crate::motion::{Animated, FrameClock};

/// What the button keeps between frames, by its id.
pub(in crate::update_button) struct UpdateMotion {
    pub(in crate::update_button) fraction: Animated,
    pub(in crate::update_button) hover: Animated,
    pub(in crate::update_button) hovered: bool,
    pub(in crate::update_button) clock: FrameClock,
    pub(in crate::update_button) focus: FocusHandle,
}
