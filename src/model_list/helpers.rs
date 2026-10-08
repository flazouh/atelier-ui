use gpui_kit::{Hsla, SharedString};

use crate::theme::{Appearance, Theme};

/// The colour of the filled star: on a dark page the theme's warning tone, which is a soft gold there; on a light page its full-strength fill,
/// since the tone is darkened to read as text on white and a mark in it is a dull brown.
pub fn star_colour(theme: &Theme) -> Hsla {
    if theme.appearance == Appearance::Dark { theme.warning } else { theme.warning_fill }
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
