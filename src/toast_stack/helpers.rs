use gpui_kit::{Hsla, IntoElement, ParentElement, Styled, div, prelude::FluentBuilder};

use crate::scale::px;
use crate::{
    icon::{Icon},
    theme::{MARK_CONTRAST, Theme, contrast, mix},
};
use super::structs::Toast;
use super::types::{
    DISC_MOST, DISC_VISIBLE, DRAG_DISTANCE, DRAG_SPEED, ELASTIC, ICON_GLYPH, MAX_WIDTH,
    SIDE_GUTTER, ToastPosition, ToastStatus,
};

/// The disc of `tone` and the glyph on it. `alpha` is where the web starts.
pub fn disc_for(theme: &Theme, tone: Hsla, alpha: f32, surface: Hsla) -> (Hsla, Hsla) {
    let mut a = alpha;
    let disc = loop {
        let disc = mix(surface, tone, a);
        if contrast(disc, surface) >= DISC_VISIBLE || a >= DISC_MOST {
            break disc;
        }
        a += 0.02;
    };
    // The glyph: the tone, moved toward the ink until it reads at 3:1 on the disc.
    let glyph = (0..=10)
        .map(|step| mix(tone, theme.foreground, step as f32 / 10.))
        .find(|glyph| contrast(*glyph, disc) >= MARK_CONTRAST)
        .unwrap_or(theme.foreground);
    (glyph, disc)
}

/// The width of the stack in a window `viewport` wide.
pub fn stack_width(viewport: f32) -> f32 {
    MAX_WIDTH.min(viewport - SIDE_GUTTER).max(0.)
}

/// Whether a drag that ended `distance` px from where it began, at `speed` px/s, dismisses the toast.
pub fn lets_go(distance: f32, speed: f32) -> bool {
    distance.abs() > DRAG_DISTANCE || speed.abs() > DRAG_SPEED
}

/// How far a toast moves for a pointer `delta` px from where the drag began.
pub fn elastic(delta: f32) -> f32 {
    delta * ELASTIC
}

/// The toasts to draw, from the top of the screen down: the newest `max` of them. A stack at the bottom lists the
/// oldest last (`flex-col-reverse`), one at the top lists it first.
pub fn drawn<T: Clone>(items: &[T], max: usize, position: ToastPosition) -> Vec<T> {
    let mut shown: Vec<T> = items[items.len().saturating_sub(max)..].to_vec();
    if position.bottom() {
        shown.reverse();
    }
    shown
}

/// One icon disc, laid over the others in the icon box: `rise` px below its place, at `opacity`.
pub(super) fn layer(theme: &Theme, toast: &Toast, spin: f32, rise: f32, opacity: f32) -> impl IntoElement {
    let (glyph, disc) = toast.status.tones(theme, theme.card);
    let icon = Icon::new(toast.status.icon()).size(px(ICON_GLYPH)).color(glyph).when(toast.status == ToastStatus::Loading, |i| i.turn(spin));
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(disc.opacity(disc.a * opacity))
        .child(div().relative().top(px(rise)).opacity(opacity).child(icon))
}
