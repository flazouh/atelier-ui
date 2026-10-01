use gpui_kit::{
    App,
    Bounds,
    InteractiveElement,
    IntoElement,
    KeyBinding,
    ParentElement,
    Pixels,
    SharedString,
    Styled,
    base::{actions::{Confirm, SelectFirst, SelectLast}},
    div,
};

use crate::scale::px;
use crate::{
    theme::{mix},
};
use super::types::{
    HEADING_HEIGHT, ITEM_DELAY, ITEM_FADE, ITEM_HEIGHT, ITEM_STEP, OPTION_INSET, PANEL_PAD,
    ROW_GAP,
};

/// A trigger's fill at `tint` (0 at rest, 1 hovered or open): the Ghost button's hover tone over `rest`.
pub(crate) fn trigger_tone(theme: &crate::Theme, rest: gpui_kit::Hsla, tint: f32) -> gpui_kit::Hsla {
    if rest.a == 0. {
        mix(rest, theme.muted_hover(), tint)
    } else {
        mix(rest, mix(rest, theme.foreground, 0.06), tint)
    }
}

/// The panel's height for `options` rows and `heading_rows` headings: the padding, the rows with 2px between them,
/// and the 1px edge.
pub fn panel_height_of(options: usize, heading_rows: usize) -> f32 {
    let rows = options + heading_rows;
    PANEL_PAD * 2. + ITEM_HEIGHT * options as f32 + HEADING_HEIGHT * heading_rows as f32 + ROW_GAP * rows.saturating_sub(1) as f32 + 2.
}

/// The option list's height: the padding, the rows with 2px between them. The surface is the trigger's height
/// plus this.
pub fn list_height_of(options: usize, heading_rows: usize) -> f32 {
    panel_height_of(options, heading_rows) - 2.
}

/// How long the last of `options` takes to come in, in seconds.
pub fn opens_in(options: usize) -> f32 {
    if options == 0 {
        return 0.;
    }
    ITEM_DELAY + ITEM_STEP * (options - 1) as f32 + ITEM_FADE
}

/// A spring of unit mass from rest toward 1, `t` seconds in: 0 at the start, 1 at rest, over 1 while it
/// overshoots. The closed form of what Motion runs for `y`.
pub fn spring_unit(stiffness: f32, damping: f32, t: f32) -> f32 {
    if t <= 0. {
        return 0.;
    }
    let w0 = stiffness.sqrt();
    let zeta = damping / (2. * w0);
    if zeta < 1. {
        let wd = w0 * (1. - zeta * zeta).sqrt();
        1. - (-zeta * w0 * t).exp() * ((wd * t).cos() + zeta * w0 / wd * (wd * t).sin())
    } else {
        1. - (-w0 * t).exp() * (1. + w0 * t)
    }
}

/// The header row's side inset at morph progress `p`: the trigger's `from` at 0, the options' at 1.
pub fn header_inset(from: f32, p: f32) -> f32 {
    from + (OPTION_INSET - from) * p.clamp(0., 1.)
}

/// The surface at morph progress `p` (0 the trigger, 1 the panel): its width and height.
pub fn surface_at(trigger: (f32, f32), panel_width: f32, list: f32, p: f32) -> (f32, f32) {
    let (tw, th) = trigger;
    ((tw + (panel_width - tw) * p).max(0.), (th + list * p).max(0.))
}

/// The keys the list adds to the base's Up, Down, Enter and Escape: Space opens and picks like Enter, Home
/// and End go to the first and last option. Call once at start, after `gpui_kit::init`.
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("space", Confirm { secondary: false }, Some("Select")),
        KeyBinding::new("home", SelectFirst, Some("Select")),
        KeyBinding::new("end", SelectLast, Some("Select")),
    ]);
}

/// Whether the trigger should light for a pointer that has not moved: nothing covers it (the list is shut and its
/// motion is over), it is not lit, and the pointer is inside it.
pub(super) fn should_light(covered: bool, hovered: bool, anchor: Option<Bounds<Pixels>>, pointer: gpui_kit::Point<Pixels>) -> bool {
    !covered && !hovered && anchor.is_some_and(|a| a.contains(&pointer))
}

/// Steps the highlighted option by `delta`, clamped to the list like native `<select>` (it does not
/// wrap). The first arrow press from no highlight lands on the first option (moving down) or the last
/// (moving up). Pure, so the stepping can be tested without a window.
pub(super) fn step_active(active: Option<usize>, delta: i32, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let next = match active {
        None if delta > 0 => 0,
        None => len - 1,
        Some(i) => (i as i32 + delta).clamp(0, len as i32 - 1) as usize,
    };
    Some(next)
}

/// The option type-ahead lands on: the first, from `current` on and wrapping, whose label starts with
/// what was typed, case aside. One letter typed again and again (`c`, `c`) goes through the options that
/// start with it, as a native list does. Pure.
pub(super) fn type_ahead(labels: &[SharedString], current: Option<usize>, typed: &str) -> Option<usize> {
    let want = typed.to_lowercase();
    if want.is_empty() || labels.is_empty() {
        return None;
    }
    let first = want.chars().next();
    let repeated = want.chars().count() > 1 && want.chars().all(|c| Some(c) == first);
    let (needle, from) = if repeated { (want.chars().take(1).collect::<String>(), current.map_or(0, |c| c + 1)) } else { (want, current.unwrap_or(0)) };
    let starts = |i: usize| labels[i].to_lowercase().starts_with(&needle);
    // A growing word stays on the current option when it still matches; a repeated letter moves on.
    (0..labels.len()).map(|k| (from + k) % labels.len()).find(|&i| starts(i))
}

/// The first letter of a label in a round of `size`, where a mark would go: the model badge's monogram.
pub(super) fn monogram(label: &SharedString, size: f32, theme: &crate::theme::Theme) -> impl IntoElement {
    let letter = crate::model_badge::monogram_letter(label);
    div()
        .debug_selector({
            let letter = letter.clone();
            move || format!("select-monogram-{letter}")
        })
        .flex()
        .flex_none()
        .size(px(size))
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(theme.card_strong)
        .text_size(px(size * 0.65))
        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
        .text_color(theme.muted_foreground)
        .child(letter)
}
