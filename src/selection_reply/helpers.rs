use super::{AddReply, DropReply, types::CONTEXT};

use gpui_kit::{App, Hsla, KeyBinding};

use crate::{badge::Tone, theme::Theme};

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

/// The fill and the ink of a badge of `tone`: a wash of its colour behind the colour itself, as a badge has it, and the
/// ink of the page for a neutral one, which must show on the box's own fill.
pub(super) fn chip_colors(tone: Tone, theme: &Theme) -> (Hsla, Hsla) {
    let wash = |c: Hsla| (c.opacity(0.14), c);
    match tone {
        Tone::Neutral => wash(theme.foreground),
        Tone::Primary => wash(theme.primary),
        Tone::Info => wash(theme.info),
        Tone::Success => wash(theme.success),
        Tone::Warning => wash(theme.warning),
        Tone::Danger => wash(theme.danger),
    }
}
