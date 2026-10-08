use gpui_kit::{AnyElement, Hsla, IntoElement, ParentElement, Styled, div, relative};

use super::super::consts::GAUGE_HEIGHT;
use crate::{
    scale::px,
    status_bar::Pressure,
    theme::Theme,
};

/// The colour of a gauge at `used`: green while there is room, amber from the status bar's warm line, red from its hot.
pub(super) fn gauge_ink(used: f32, theme: &Theme) -> Hsla {
    match Pressure::of(used) {
        Pressure::Calm => theme.success,
        Pressure::Warm => theme.warning,
        Pressure::Hot => theme.danger,
    }
}

/// A thin track with a fill to `used`, full width.
pub(super) fn gauge(used: f32, theme: &Theme) -> AnyElement {
    let used = if used.is_finite() { used.clamp(0., 1.) } else { 0. };
    div()
        .w_full()
        .h(px(GAUGE_HEIGHT))
        .rounded_full()
        .bg(theme.foreground.opacity(0.1))
        .child(div().h_full().w(relative(used)).rounded_full().bg(gauge_ink(used, theme)))
        .into_any_element()
}
