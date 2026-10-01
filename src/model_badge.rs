//! The model a turn or a subagent uses, such as "Opus 5.5": small muted text with no fill, so it stays
//! quieter than a status [`crate::badge::Badge`] in a dense header. It may lead with the lab's mark.
//!
//! The mark is data ([`BrandMark`]): atelier-ui names no lab, and an agent crate hands the asset paths over. It
//! sits before the label at the label's size, in its own colours, as acepe draws it: grey and at half
//! opacity at rest, taking its colour when the pointer is on the badge, or on its row when the owner says
//! so ([`ModelBadge::lit`]), over [`duration::REVEAL`]. Under Reduce Motion it switches at once. With no
//! mark, a monogram of the label's first letter stands in its place.

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, StyledImage,
    Window, div, img, 
};
use crate::scale::px;

use crate::{
    motion::{Channel, Curve, duration, ease},
    theme::{ActiveTheme, Appearance},
};

/// A lab's or an agent's mark, one asset for each theme.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrandMark {
    pub light: SharedString,
    pub dark: SharedString,
}

impl BrandMark {
    pub fn new(light: impl Into<SharedString>, dark: impl Into<SharedString>) -> Self {
        Self { light: light.into(), dark: dark.into() }
    }

    /// The asset for the theme in force.
    pub fn for_theme(&self, appearance: Appearance) -> SharedString {
        match appearance {
            Appearance::Light => self.light.clone(),
            Appearance::Dark => self.dark.clone(),
        }
    }
}

/// The mark's size: the label's own.
const MARK: f32 = 11.;

#[derive(IntoElement)]
pub struct ModelBadge {
    id: Option<ElementId>,
    label: SharedString,
    mark: Option<BrandMark>,
    lit: bool,
}

impl ModelBadge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { id: None, label: label.into(), mark: None, lit: false }
    }

    /// Leads with a mark. It needs an id, which keeps its fade from one frame to the next.
    pub fn mark(mut self, id: impl Into<ElementId>, mark: BrandMark) -> Self {
        self.id = Some(id.into());
        self.mark = Some(mark);
        self
    }

    /// A monogram in the mark's place, for a model with no known lab.
    pub fn monogram(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// The row the badge sits in is under the pointer, so the mark takes its colour.
    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }
}

/// The badge's own hover, and how far its mark has come into colour.
struct MarkMotion {
    hovered: bool,
    colour: Channel,
}

/// The monogram's letter: the label's first, capitalised.
pub fn monogram_letter(label: &str) -> SharedString {
    label.chars().next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_else(|| "?".into()).into()
}

impl RenderOnce for ModelBadge {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let label = div()
            .flex_none()
            .whitespace_nowrap()
            .text_size(px(11.))
            .line_height(px(16.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(muted.opacity(0.8))
            .child(self.label.clone());
        let Some(id) = self.id else { return label.into_any_element() };

        let reduce = cx.reduce_motion();
        let motion = window.use_keyed_state(id.clone(), cx, |_, _| MarkMotion { hovered: false, colour: Channel::new(0.) });
        let want = if self.lit || motion.read(cx).hovered { 1. } else { 0. };
        motion.update(cx, |m, _| {
            if (m.colour.target() - want).abs() > 1e-3 {
                m.colour.animate(want, Curve::Ease(duration::REVEAL.as_secs_f32(), ease::OUT), 0., reduce);
            }
        });
        let p = {
            let m = motion.read(cx);
            if m.colour.is_running() {
                window.request_animation_frame();
            }
            m.colour.value().clamp(0., 1.)
        };
        // The grey copy at half opacity fades out as the coloured one fades in: `img` greyscale is on or
        // off, so a cross-fade is how it moves.
        let mark = match &self.mark {
            Some(mark) => {
                let path = mark.for_theme(theme.appearance);
                div()
                    .relative()
                    .flex_none()
                    .size(px(MARK))
                    .child(img(path.clone()).absolute().inset_0().size(px(MARK)).grayscale(true).opacity(0.5 * (1. - p)))
                    .child(img(path).absolute().inset_0().size(px(MARK)).opacity(p))
                    .into_any_element()
            }
            None => div()
                .flex()
                .flex_none()
                .size(px(MARK))
                .items_center()
                .justify_center()
                .text_size(px(9.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(crate::theme::mix(muted.opacity(0.5), theme.foreground, p))
                .child(monogram_letter(&self.label))
                .into_any_element(),
        };
        let hover = motion.clone();
        div()
            .id(id)
            .flex()
            .flex_none()
            .items_center()
            .gap(px(5.))
            .on_hover(move |on, _, cx| {
                hover.update(cx, |m, cx| {
                    m.hovered = *on;
                    cx.notify();
                })
            })
            .child(mark)
            .child(label)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests;
