use super::{AddReply, DropReply, types::CONTEXT};

use gpui_kit::{App, KeyBinding};

/// Inside the box, Enter adds the reply and Escape drops it. They are bound a level deeper than the input's own
/// keys, so they win; Shift-Enter stays the input's new line.
pub(crate) fn bind_keys(cx: &mut App) {
    let context = Some(format!("{CONTEXT} > Input"));
    let context = context.as_deref();
    cx.bind_keys([
        KeyBinding::new("enter", AddReply, context),
        KeyBinding::new("secondary-enter", AddReply, context),
        KeyBinding::new("escape", DropReply, context),
    ]);
}

/// `text` cut to `most` characters, with an ellipsis when it was cut, and its lines joined by a space.
pub(super) fn shown(text: &str, most: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= most {
        return flat;
    }
    let cut: String = flat.chars().take(most).collect();
    format!("{}…", cut.trim_end())
}
