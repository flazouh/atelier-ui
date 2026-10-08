use gpui_kit::{Hsla, SharedString};

use crate::theme::{Appearance, Theme};

/// The colour of the filled star. On a dark page it is the theme's `warning` tone, a soft gold there. On a light page the tone is darkened to read
/// as text on white (a dull brown), and a theme's own fill can be any warning colour (Cursor Light's is a vermilion), so the star takes the design
/// system's gold: the full amber of the atelier theme.
pub fn star_colour(theme: &Theme) -> Hsla {
    if theme.appearance == Appearance::Dark { theme.warning } else { crate::themes::atelier(Appearance::Light).warning_fill }
}

/// `order` with `id` taken out and put before `before` (at the end for none). An id not in the order, or dropped on itself, changes nothing.
pub fn moved(order: &[SharedString], id: &SharedString, before: Option<&SharedString>) -> Vec<SharedString> {
    if !order.contains(id) || before == Some(id) {
        return order.to_vec();
    }
    let mut rest: Vec<SharedString> = order.iter().filter(|o| *o != id).cloned().collect();
    let at = before.and_then(|b| rest.iter().position(|o| o == b)).unwrap_or(rest.len());
    rest.insert(at, id.clone());
    rest
}
