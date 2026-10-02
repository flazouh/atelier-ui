use gpui_kit::Hsla;

use crate::theme::Theme;
use super::structs::{Inset, MenuLook};
use super::types::{BORDER, CLIP_HALF, FILL_RAMP, LINE, RADIUS_END, RADIUS_START, Tone};

/// The height of a panel of `rows` plain rows in `look`: the padding, the rows and the 1px edge.
pub fn height_in(look: MenuLook, rows: usize) -> f32 {
    2. * look.pad + rows as f32 * (LINE + 2. * look.row_y) + 2.
}

/// The height in the default look.
pub fn height(rows: usize) -> f32 {
    height_in(MenuLook::BAR, rows)
}

/// The clip at the start: a square 16px across at `origin` (CSS `collapsedClip`).
pub fn collapsed(origin: (f32, f32), size: (f32, f32)) -> Inset {
    Inset {
        top: (origin.1 - CLIP_HALF).clamp(0., size.1),
        right: (size.0 - origin.0 - CLIP_HALF).clamp(0., size.0),
        bottom: (size.1 - origin.1 - CLIP_HALF).clamp(0., size.1),
        left: (origin.0 - CLIP_HALF).clamp(0., size.0),
    }
}

/// The clip at unfold progress `t` (0 to 1) and its corner radius.
pub fn unfolded(start: Inset, t: f32) -> (Inset, f32) {
    let k = 1. - t;
    (
        Inset { top: start.top * k, right: start.right * k, bottom: start.bottom * k, left: start.left * k },
        RADIUS_START + (RADIUS_END - RADIUS_START) * t,
    )
}

/// The whole panel's size from what its probe measured inside the border.
pub fn panel_size(inner_width: f32, inner_height: f32) -> (f32, f32) {
    (inner_width + 2. * BORDER, inner_height + 2. * BORDER)
}

/// The panel's opacity while it unfolds, `reveal` being how far the unfold has gone (0 to 1).
pub fn fill_opacity(reveal: f32) -> f32 {
    (reveal * FILL_RAMP).clamp(0., 1.)
}

/// The row that Up (`step` 1) or Down (`step` -1) reaches from `current`, among the rows in `reachable`. It
/// wraps. With no current row, the first.
pub fn walk(reachable: &[usize], current: Option<usize>, step: i32) -> Option<usize> {
    if reachable.is_empty() {
        return None;
    }
    let at = current.and_then(|c| reachable.iter().position(|r| *r == c));
    let next = match at {
        None => 0,
        Some(at) => (at as i32 + step).rem_euclid(reachable.len() as i32) as usize,
    };
    Some(reachable[next])
}

/// The first of `rows` (an index and its words) whose words start with `typed`, in any case.
pub fn jump(rows: &[(usize, String)], typed: &str) -> Option<usize> {
    let typed = typed.to_lowercase();
    rows.iter().find(|(_, words)| words.trim().to_lowercase().starts_with(&typed)).map(|(i, _)| *i)
}

pub(super) fn pill_fill(theme: &Theme, tone: Tone) -> Hsla {
    match tone {
        Tone::Default => theme.foreground.opacity(0.065),
        Tone::Destructive => theme.danger.opacity(0.10),
    }
}

/// The panel's shadow: the popover's, times the site's strength.
pub(super) fn panel_shadow(theme: &Theme, strength: f32) -> Vec<gpui_kit::BoxShadow> {
    crate::theme::popover_shadow(theme)
        .into_iter()
        .map(|mut shadow| {
            shadow.color = shadow.color.opacity(strength);
            shadow
        })
        .collect()
}
