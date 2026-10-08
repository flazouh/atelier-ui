use gpui_kit::{App, InteractiveElement, ParentElement, Pixels, Styled, Window};

use crate::theme::ActiveTheme;
use super::helpers::row_ring;

/// Makes a pressable element a stop in the Tab order that shows the house ring while the keyboard has it, so a row, a
/// header or a link that a press opens can be reached and opened without the pointer. Enter or Space presses it: gpui
/// turns them into a click on a focused element that has an `on_click`. Put it before the `on_click`.
///
/// `key` names the element's focus for as long as it is drawn; it must differ from every other pressable's.
pub trait PressStop: InteractiveElement + ParentElement + Styled + Sized {
    fn press_stop(self, key: impl Into<gpui_kit::ElementId>, radius: Pixels, window: &mut Window, cx: &mut App) -> Self {
        let theme = cx.theme().clone();
        let handle = window.use_keyed_state(key.into(), cx, |_, cx| cx.focus_handle()).read(cx).clone();
        let keyed = handle.is_focused(window) && window.last_input_was_keyboard();
        let el = self.relative().track_focus(&handle.tab_stop(true));
        if keyed { el.child(row_ring(&theme, theme.background, radius)) } else { el }
    }
}

impl<T: InteractiveElement + ParentElement + Styled + Sized> PressStop for T {}
