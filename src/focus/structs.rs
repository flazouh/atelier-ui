use gpui_kit::{
    AnyElement, App, FocusHandle, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, Styled, Window, div, prelude::FluentBuilder,
};

use crate::theme::{ActiveTheme, radius};
use super::helpers::ring_shadow;

/// A text field's box: a fill and a corner, and the ring while focus is inside it.
#[derive(IntoElement)]
pub struct Field {
    pub(super) focus: FocusHandle,
    pub(super) child: AnyElement,
    pub(super) radius: Pixels,
    pub(super) surface: Option<Hsla>,
    pub(super) padding: Option<Pixels>,
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
