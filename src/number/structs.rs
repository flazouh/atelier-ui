use gpui_kit::{
    App,
    ElementId,
    Hsla,
    IntoElement,
    ParentElement,
    Pixels,
    RenderOnce,
    SharedString,
    Styled,
    Window,
    div,
};

use crate::scale::px;
use crate::{
    roll::{Kind, Roll},
    theme::ActiveTheme,
};
use super::types::SLOT_WIDTH;
use super::helpers::{line_for, runs};

#[derive(IntoElement)]
pub struct Digits {
    id: ElementId,
    pub(super) text: SharedString,
    pub(super) size: Pixels,
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
