use gpui_kit::{App, Hsla, IntoElement, Pixels, RenderOnce, Styled, Window, canvas};

use super::helpers::paint;

/// What the mark shows, with the amounts that animate between states.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub color: Hsla,
    /// The circle outline, and whether it is dashed (pending).
    pub ring_alpha: f32,
    pub dashed: bool,
    /// Fill wash inside the ring (completed uses 6%).
    pub fill_alpha: f32,
    /// How much of the progress arc is drawn, from 0 to 1, and where it starts in turns.
    pub arc: f32,
    pub arc_start: f32,
    /// How much of the check, the cross, and the slash is drawn, from 0 to 1.
    pub check: f32,
    pub cross: f32,
    pub slash: f32,
    /// The color of the check, cross, or slash. `None` draws them in `color`; `Some(page)` knocks them
    /// out of a filled disc in the page color.
    pub glyph: Option<Hsla>,
}

impl Mark {
    /// A quiet mark: nothing drawn.
    pub fn none(color: Hsla) -> Self {
        Self {
            color,
            ring_alpha: 0.,
            dashed: false,
            fill_alpha: 0.,
            arc: 0.,
            arc_start: 0.,
            check: 0.,
            cross: 0.,
            slash: 0.,
            glyph: None,
        }
    }

    /// A solid disc in `color`, with any glyph knocked out in `page`.
    pub fn filled(color: Hsla, page: Hsla) -> Self {
        Self {
            fill_alpha: 1.,
            glyph: Some(page),
            ..Self::none(color)
        }
    }

    pub fn check(mut self, amount: f32) -> Self {
        self.check = amount;
        self
    }

    pub fn cross(mut self, amount: f32) -> Self {
        self.cross = amount;
        self
    }

    pub fn slash(mut self, amount: f32) -> Self {
        self.slash = amount;
        self
    }
}

#[derive(IntoElement)]
pub struct StatusMark {
    pub(super) mark: Mark,
    pub(super) size: Pixels,
}

impl StatusMark {
    pub fn new(mark: Mark, size: Pixels) -> Self {
        Self { mark, size }
    }
}

impl RenderOnce for StatusMark {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mark = self.mark;
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| paint(bounds, mark, window),
        )
        .size(self.size)
        .flex_none()
    }
}
