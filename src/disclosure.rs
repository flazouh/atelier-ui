//! A header row that opens and closes its body, like beui's agent disclosure. Tool calls, diffs, and
//! thinking use it. The chevron points right when closed and down when open, and the body fades in;
//! with Reduce Motion the fade is skipped.

use std::sync::Arc;

use crate::scale::px;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, App, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};

use crate::{
    focus::PressStop,
    icon::{Icon, IconName},
    motion::{cubic_bezier, duration, ease},
    theme::{ActiveTheme, radius},
};

#[derive(IntoElement)]
pub struct Disclosure {
    id: ElementId,
    header: AnyElement,
    body: Option<AnyElement>,
    default_open: bool,
}

impl Disclosure {
    pub fn new(id: impl Into<ElementId>, header: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            header: header.into_any_element(),
            body: None,
            default_open: false,
        }
    }

    /// What the header reveals. Without a body the row shows no chevron and does not open.
    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }

    /// Opens on first render. The user's choice wins after that.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }
}

impl RenderOnce for Disclosure {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let default_open = self.default_open;
        let open_state = window.use_keyed_state(self.id.clone(), cx, move |_, _| default_open);
        let open = *open_state.read(cx);
        let theme = cx.theme();
        let (faint, fg) = (theme.faint(), theme.foreground);
        let has_body = self.body.is_some();
        let child_id =
            |name: &'static str| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());

        let header = div()
            .id(child_id("header"))
            .group("disclosure-header")
            .flex()
            .items_center()
            .gap(px(8.))
            .min_h(px(36.))
            .py(px(4.))
            .rounded(radius::md())
            .child(div().flex_1().min_w_0().child(self.header))
            .when(has_body, |d| {
                d.cursor_pointer()
                    .press_stop(child_id("press"), crate::theme::radius::md(), window, cx)
                    .on_click(move |_, _, cx| {
                        open_state.update(cx, |open, cx| {
                            *open = !*open;
                            cx.notify();
                        })
                    })
                    .child(
                        div()
                            .flex_none()
                            .text_color(faint)
                            .group_hover("disclosure-header", |s| s.text_color(fg))
                            .child(
                                Icon::new(if open {
                                    IconName::ChevronDown
                                } else {
                                    IconName::ChevronRight
                                })
                                .size(px(14.)),
                            ),
                    )
            });

        let reduce_motion = cx.reduce_motion();
        let body_id = child_id("body");
        div().flex().flex_col().w_full().child(header).when_some(
            self.body.filter(|_| open),
            |d, body| {
                let body = div().pl(px(24.)).pt(px(6.)).child(body);
                if reduce_motion {
                    d.child(body)
                } else {
                    d.child(
                        body.with_animation(
                            body_id,
                            Animation::new(duration::REVEAL)
                                .with_easing(|t| cubic_bezier(ease::OUT, t)),
                            |body, t| body.opacity(t),
                        ),
                    )
                }
            },
        )
    }
}
