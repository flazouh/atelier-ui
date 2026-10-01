//! Digits: beui.dev's Digit Swap (`components/motion/digit-swap.tsx`), from Number Animation. A number in fixed
//! slots: when a digit changes, the old one leaves through the top and the new one comes up from 45% below,
//! clear to full over 180ms ([`crate::roll::Kind::Digit`]). A digit that did not change does not move.
//!
//! [`Digits`] takes any text and gives every run of digits its slots, one per character, `1ch` wide (0.6em in
//! the monospace font, the same in the text font, whose digits differ little), as tall as the line of the text around them and clipped. The
//! rest of the text is plain, so "3 of 12 reviewed" rolls its two numbers and keeps its words still.
//!
//! Under Reduce Motion a digit changes at once. What gpui cannot draw is left out: the 6ms stagger from one
//! slot to the next (a slot keeps its place, not its delay).
use gpui_kit::{App, ElementId, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled, Window, div, };
use crate::scale::px;

use crate::{
    roll::{Kind, Roll},
    theme::ActiveTheme,
};

/// A slot's width, as a share of the text size (`1ch` of the monospace font), and its height (`h-[1.1em]`).
pub const SLOT_WIDTH: f32 = 0.6;
/// The line height that goes with a text size, on the 4px grid: 12 is 16, 14 is 20, 16 is 24.
pub fn line_for(size: f32) -> f32 {
    (size * 1.4 / 4.).round() * 4.
}

/// The text cut into runs: `(text, is_digits)`.
pub fn runs(text: &str) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = Vec::new();
    for c in text.chars() {
        let digit = c.is_ascii_digit();
        match out.last_mut() {
            Some((run, d)) if *d == digit => run.push(c),
            _ => out.push((c.to_string(), digit)),
        }
    }
    out
}

#[derive(IntoElement)]
pub struct Digits {
    id: ElementId,
    text: SharedString,
    size: Pixels,
    color: Option<Hsla>,
}

impl Digits {
    /// `size` is the text size the slots are measured in.
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>, size: Pixels) -> Self {
        Self { id: id.into(), text: text.into(), size, color: None }
    }

    /// The colour of the digits; the surrounding text colour by default.
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl RenderOnce for Digits {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let size = f32::from(self.size);
        let (slot, tall) = (SLOT_WIDTH * size, line_for(size));
        let _ = cx.theme();
        let mut at = 0usize;
        div()
            .flex()
            .flex_none()
            .items_center()
            .h(px(tall))
            .line_height(px(tall))
            .whitespace_nowrap()
            .text_size(self.size)
            .children(runs(&self.text).into_iter().flat_map(|(run, digits)| {
                if digits {
                    run.chars()
                        .map(|c| {
                            at += 1;
                            let key = at;
                            Roll::new(ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), format!("digit-{key}").into()), c, Kind::Digit, px(tall), move |c: &char| {
                                div().w(px(slot)).h(px(tall)).flex().items_center().justify_center().line_height(px(tall)).child(c.to_string()).into_any_element()
                            })
                            .into_any_element()
                        })
                        .collect::<Vec<_>>()
                } else {
                    at += run.chars().count();
                    vec![div().flex_none().child(run).into_any_element()]
                }
            }))
    }
}

#[cfg(test)]
mod tests;
