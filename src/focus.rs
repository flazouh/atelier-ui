//! One focus style for every field, button and row that the app owns: a 2px ring just outside the control, in the
//! ink at the least strength that reaches 3:1 on the surface behind it. It is quiet in a theme with a light page and
//! in one with a dark page alike. It is drawn for keyboard focus on a button and row, and for any focus inside a
//! [`Field`] (a text field shows where the caret is, whoever put it there).
//!
//! The parts ported from beui.dev keep the focus look of the web version (a checkbox, a slider, a swatch).
use gpui_kit::{
    AnyElement, App, BoxShadow, FocusHandle, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window, div, point,
    prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::theme::{ActiveTheme, MARK_CONTRAST, Theme, contrast, mix, radius};

/// How thick the ring is.
pub const RING_WIDTH: f32 = 2.;

/// The ring's colour on `surface`: the ink, mixed toward `surface` no further than it must be for 3:1.
pub fn ring_color(theme: &Theme, surface: Hsla) -> Hsla {
    (1..=20)
        .map(|step| mix(surface, theme.foreground, step as f32 / 20.))
        .find(|ring| contrast(*ring, surface) >= MARK_CONTRAST)
        .unwrap_or(theme.foreground)
}

/// The ring as a shadow with no blur and a spread of [`RING_WIDTH`], drawn outside the control's box.
pub fn ring_shadow(theme: &Theme, surface: Hsla) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: ring_color(theme, surface),
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(RING_WIDTH),
        inset: false,
    }]
}

/// The ring for a row that has the keyboard cursor: the same 2px in the same colour, drawn inside the row's edge, so a
/// neighbour or a scroll box does not clip it. Put it as the last child of a `relative` row of the same `radius`.
pub fn row_ring(theme: &Theme, surface: Hsla, radius: Pixels) -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .rounded(radius)
        .border(px(RING_WIDTH))
        .border_color(ring_color(theme, surface))
        .debug_selector(|| "row-ring".into())
}

/// Makes a pressable element a stop in the Tab order that shows the house ring while the keyboard has it, so a row, a
/// header or a link that a press opens can be reached and opened without the pointer. Enter or Space presses it: gpui
/// turns them into a click on a focused element that has an `on_click`. Put it before the `on_click`.
///
/// `key` names the element's focus for as long as it is drawn; it must differ from every other pressable's.
pub trait PressStop: InteractiveElement + ParentElement + Styled + Sized {
    fn press_stop(self, key: impl Into<gpui_kit::ElementId>, radius: Pixels, window: &mut Window, cx: &mut App) -> Self {
        let theme = cx.theme().clone();
        let handle = window.use_keyed_state(key.into(), cx, |_, cx| cx.focus_handle()).read(cx).clone();
        let keyed = handle.is_focused(window) && window.last_input_was_keyboard();
        let el = self.relative().track_focus(&handle.tab_stop(true));
        if keyed { el.child(row_ring(&theme, theme.background, radius)) } else { el }
    }
}

impl<T: InteractiveElement + ParentElement + Styled + Sized> PressStop for T {}

/// A text field's box: a fill and a corner, and the ring while focus is inside it.
#[derive(IntoElement)]
pub struct Field {
    focus: FocusHandle,
    child: AnyElement,
    radius: Pixels,
    surface: Option<Hsla>,
    padding: Option<Pixels>,
}

impl Field {
    /// A box round `child`, which shows the ring while `focus` (the input's handle) or a part inside it has focus.
    pub fn new(focus: FocusHandle, child: impl IntoElement) -> Self {
        Self { focus, child: child.into_any_element(), radius: radius::md(), surface: None, padding: None }
    }

    pub fn radius(mut self, radius: Pixels) -> Self {
        self.radius = radius;
        self
    }

    /// The fill; the theme's `card_strong` by default.
    pub fn surface(mut self, surface: Hsla) -> Self {
        self.surface = Some(surface);
        self
    }

    pub fn padding(mut self, padding: Pixels) -> Self {
        self.padding = Some(padding);
        self
    }
}

impl RenderOnce for Field {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let surface = self.surface.unwrap_or(theme.card_strong);
        let focused = self.focus.contains_focused(window, cx);
        div()
            .rounded(self.radius)
            .bg(surface)
            .when_some(self.padding, |d, p| d.p(p))
            .when(focused, |d| d.shadow(ring_shadow(&theme, surface)).debug_selector(|| "field-ring".into()))
            .child(self.child)
    }
}

#[cfg(test)]
mod tests;
