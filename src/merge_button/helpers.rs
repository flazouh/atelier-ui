use gpui_kit::{App, Entity, Window};

use super::structs::MenuState;

/// Closes the menu and, when the keyboard closed it, gives focus back to the arrow.
pub(super) fn close_menu(menu: &Entity<MenuState>, from_keys: bool, window: &mut Window, cx: &mut App) {
    let arrow = menu.update(cx, |m, cx| {
        m.set_open(false);
        cx.notify();
        m.arrow.clone()
    });
    if from_keys {
        window.focus(&arrow, cx);
    }
}
