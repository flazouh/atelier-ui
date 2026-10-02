use super::types::CHECK_FROM;

/// The scroll offset (from the top, as a positive number) that brings a row in `top..bottom` into a view
/// `view_top..view_bottom` that is scrolled by `offset`, or `None` when it is in sight already.
pub fn scroll_to_show(offset: f32, view_height: f32, top: f32, bottom: f32) -> Option<f32> {
    if top < offset {
        Some(top)
    } else if bottom > offset + view_height {
        Some(bottom - view_height)
    } else {
        None
    }
}

/// The check's scale and opacity at progress `t` of its appearance.
pub fn check_at(t: f32) -> (f32, f32) {
    (CHECK_FROM + (1. - CHECK_FROM) * t, t)
}
