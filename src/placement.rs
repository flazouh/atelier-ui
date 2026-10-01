//! Where a popover opens: below its anchor, or above it when the space below is too short for the
//! panel and the space above is larger. Select and the merge menu both ask here, so a picker at the
//! foot of a pane and a merge button near the bottom of the rail turn over the same way.
//!
//! The anchor's bounds come from its last layout ([`measure`]); a panel opens on a press, a frame
//! after that layout, so the bounds are always there when the question is asked.

use gpui_kit::{App, Bounds, IntoElement, Pixels, Styled, canvas};

/// Whether a panel `panel` tall, `gap` away from `anchor`, opens above it in a window `window` tall.
pub fn opens_upward(anchor: Bounds<Pixels>, panel: f32, gap: f32, window: f32) -> bool {
    let below = window - f32::from(anchor.bottom()) - gap;
    let above = f32::from(anchor.top()) - gap;
    below < panel && above > below
}

/// A canvas over its parent that reports the parent's bounds after layout.
pub(crate) fn measure(report: impl Fn(Bounds<Pixels>, &mut App) + 'static) -> impl IntoElement {
    canvas(move |bounds, _, cx| report(bounds, cx), |_, _, _, _| {}).absolute().inset_0()
}

#[cfg(test)]
mod tests;
