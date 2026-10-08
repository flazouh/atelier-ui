use gpui_kit::{Hsla, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div};

use super::structs::{Inset, MenuItem, MenuLook};
use super::types::{
    BORDER, Branch, CLIP_HALF, Entry, FILL_RAMP, LABEL, LEAD, LINE, Lead, Pick, RADIUS_END,
    RADIUS_START, SLOT, Tone,
};
use crate::scale::px;
use crate::theme::Theme;

/// The height of a panel of `rows` plain rows in `look`: the padding, the rows and the 1px edge.
pub fn height_in(look: MenuLook, rows: usize) -> f32 {
    2. * look.pad + rows as f32 * (LINE + 2. * look.row_y) + 2.
}

/// The height of a panel of `entries` in `look`: the rows, each heading over a group, and the gap between groups.
pub fn height_of(look: MenuLook, entries: &[super::types::Entry]) -> f32 {
    use super::types::Entry;
    let rows = entries
        .iter()
        .filter(|e| matches!(e, Entry::Item(_)))
        .count();
    entries.iter().fold(height_in(look, rows), |h, e| match e {
        Entry::Label(_) => h + LABEL,
        Entry::Separator => h + look.group,
        Entry::Item(_) => h,
    })
}

/// A lead of `size`: the mark's image, or the monogram of `label`. For any place an agent's name shows.
pub fn lead_icon(
    label: &SharedString,
    lead: Lead,
    size: f32,
    theme: &Theme,
) -> gpui_kit::AnyElement {
    match lead {
        Lead::Mark(mark) => gpui_kit::img(mark.for_theme(theme.appearance))
            .flex_none()
            .size(px(size))
            .into_any_element(),
        Lead::Monogram => crate::select::monogram(label, size, theme).into_any_element(),
    }
}

/// The slot before a row's words that holds its lead: a mark's image, or the monogram of the words.
pub(super) fn lead_slot(label: &SharedString, lead: Lead, theme: &Theme) -> gpui_kit::AnyElement {
    let inner = lead_icon(label, lead, LEAD, theme);
    let name = format!("menu-lead-{label}");
    div()
        .debug_selector(move || name.clone())
        .flex_none()
        .mt(px(2.))
        .size(px(SLOT))
        .flex()
        .items_center()
        .justify_center()
        .child(inner)
        .into_any_element()
}

/// The rows of a menu for `branches`: a leaf is a row that runs `pick` with its id, a branch a row that opens the
/// menu of its own branches. Every row is named `branch-<id>` for tests.
pub fn entries_of(branches: &[Branch], pick: &Pick) -> Vec<Entry> {
    branches
        .iter()
        .map(|branch| {
            let row =
                MenuItem::new(branch.label.clone()).debug_name(format!("branch-{}", branch.id));
            let row = match &branch.lead {
                Some(lead) => row.lead(lead.clone()),
                None => row,
            };
            if branch.branches.is_empty() {
                let (pick, id) = (pick.clone(), branch.id.clone());
                row.on_select(move |window, cx| pick(&id, window, cx))
                    .into()
            } else {
                row.submenu(entries_of(&branch.branches, pick)).into()
            }
        })
        .collect()
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
        Inset {
            top: start.top * k,
            right: start.right * k,
            bottom: start.bottom * k,
            left: start.left * k,
        },
        RADIUS_START + (RADIUS_END - RADIUS_START) * t,
    )
}

/// The whole panel's size, in design pixels, from what its probe measured inside the border, in window pixels. The border
/// is a real pixel wide at any zoom, so it is added before the zoom is taken out: add it after, and each frame the panel
/// would be a little wider than the last.
pub fn panel_size(inner_width: gpui_kit::Pixels, inner_height: gpui_kit::Pixels) -> (f32, f32) {
    let border = gpui_kit::px(2. * BORDER);
    (
        crate::scale::design(inner_width + border),
        crate::scale::design(inner_height + border),
    )
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
    rows.iter()
        .find(|(_, words)| words.trim().to_lowercase().starts_with(&typed))
        .map(|(i, _)| *i)
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
