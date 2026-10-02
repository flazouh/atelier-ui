use std::sync::Arc;

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    agent_look::AgentLook,
    focus::PressStop,
    icon::{Icon, IconName},
    model_badge::{BrandMark, ModelBadge},
    morph::Morph,
    reveal::Reveal,
    spinner::Spinner,
    status_mark::{Mark, StatusMark},
    theme::{ActiveTheme, StatusTone, radius},
    tool_call::ToolCall,
    typography::{SEGMENT_GAP, TextSize},
};
use super::helpers::status_line;

#[derive(IntoElement)]
pub struct SubagentCard {
    id: ElementId,
    pub(super) look: AgentLook,
    pub(super) name: SharedString,
    pub(super) task: SharedString,
    model: Option<SharedString>,
    model_mark: Option<BrandMark>,
    pub(super) elapsed: Option<SharedString>,
    pub(super) tool_calls: u64,
    live_tool: Option<SharedString>,
    pub(super) finished: Option<Option<u64>>,
    pub(super) calls: Vec<ToolCall>,
}

impl SubagentCard {
    pub fn new(
        id: impl Into<ElementId>,
        look: AgentLook,
        name: impl Into<SharedString>,
        task: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            look,
            name: name.into(),
            task: task.into(),
            model: None,
            model_mark: None,
            elapsed: None,
            tool_calls: 0,
            live_tool: None,
            finished: None,
            calls: Vec::new(),
        }
    }

    /// The model it runs on, such as "Opus 5.5".
    /// The lab's mark before the model's name. It takes its colour while the pointer is on the header.
    pub fn model_mark(mut self, mark: BrandMark) -> Self {
        self.model_mark = Some(mark);
        self
    }

    pub fn model(mut self, model: impl Into<SharedString>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// How long it has run, such as "12s".
    pub fn elapsed(mut self, elapsed: impl Into<SharedString>) -> Self {
        self.elapsed = Some(elapsed.into());
        self
    }

    pub fn tool_calls(mut self, count: u64) -> Self {
        self.tool_calls = count;
        self
    }

    /// The tool call running now, such as "Read crates/beui/src/theme.rs".
    pub fn live_tool(mut self, tool: impl Into<SharedString>) -> Self {
        self.live_tool = Some(tool.into());
        self
    }

    /// Marks it done, with its run time in seconds if known.
    pub fn finished(mut self, seconds: Option<u64>) -> Self {
        self.finished = Some(seconds);
        self
    }

    /// Its tool calls, shown when the card opens.
    pub fn calls(mut self, calls: Vec<ToolCall>) -> Self {
        self.calls = calls;
        self
    }
}

impl RenderOnce for SubagentCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let done = self.finished.is_some();
        let has_calls = !self.calls.is_empty();
        let disclosure = window.use_keyed_state(self.id.clone(), cx, |_, _| Reveal::new(false));
        let header_hover = window.use_keyed_state(ElementId::NamedChild(Arc::new(self.id.clone()), "hover".into()), cx, |_, _| false);
        let lit = *header_hover.read(cx);
        if disclosure.read(cx).is_moving() {
            window.request_animation_frame();
        }
        let (reveal, chevron) = {
            let d = disclosure.read(cx);
            (d.reveal.value(), d.chevron.value())
        };
        let child = |name: &'static str| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());
        let mark = &self.look.mark;
        let text = |text: SharedString| move |_: &mut Window, _: &mut App| div().truncate().child(text.clone()).into_any_element();

        let right = if done {
            StatusMark::new(Mark::filled(theme.status_tone(StatusTone::Done), theme.background).check(1.), px(14.))
                .into_any_element()
        } else {
            div().text_size(TextSize::Xs.font_size()).text_color(muted.opacity(0.7)).children(self.elapsed.clone()).into_any_element()
        };
        let toggle = disclosure.clone();
        let header = div()
            .id(child("header"))
            .on_hover({
                let hover = header_hover.clone();
                move |on, _, cx| hover.update(cx, |h, cx| {
                    *h = *on;
                    cx.notify();
                })
            })
            .group("subagent-header")
            .flex()
            .flex_col()
            .gap(px(2.))
            .px(px(14.))
            .py(px(10.))
            .rounded(radius::xxl())
            .when(has_calls, |d| {
                d.cursor_pointer().press_stop((self.id.clone(), "head-focus"), crate::theme::radius::md(), window, cx).on_click(move |_, _, cx| {
                    let reduce = cx.reduce_motion();
                    toggle.update(cx, |d, cx| {
                        let open = !d.open;
                        d.set_open(open, reduce);
                        cx.notify();
                    })
                })
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .h(px(24.))
                    .child(mark.sprite(child("mark"), mark.orbiting).size(px(16.)).playing(!done))
                    .child(
                        div()
                            .flex_none()
                            .text_size(TextSize::Sm.font_size())
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if done { theme.foreground.opacity(0.9) } else { self.look.message })
                            .child(self.name),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(TextSize::Sm.font_size())
                            .text_color(muted)
                            .child(self.task),
                    )
                    .children(self.model.map(|model| {
                        let badge = ModelBadge::new(model).lit(lit);
                        match self.model_mark {
                            Some(mark) => badge.mark(child("model"), mark),
                            None => badge,
                        }
                    }))
                    .child(div().flex_none().flex().items_center().child(right))
                    .when(has_calls, |d| {
                        d.child(
                            div()
                                .flex_none()
                                .text_color(muted.opacity(0.5))
                                .group_hover("subagent-header", |s| s.text_color(muted))
                                .child(Icon::new(IconName::ChevronDown).size(px(14.)).turn(chevron / 360.)),
                        )
                    }),
            )
            .when(done || self.tool_calls > 0 || self.live_tool.is_some(), |d| d.child(
                // Under the name: past the mark and its gap. Before the first tool call there is nothing
                // to count, so the header stands alone.
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .pl(px(26.))
                    .h(px(20.))
                    .text_size(TextSize::Xs.font_size())
                    .text_color(muted)
                    .child(div().flex_none().child({
                        let segments = status_line(self.finished, self.tool_calls);
                        let key = segments.join("\n");
                        Morph::new(child("count"), key, move |_, _| {
                            div().flex().gap(px(SEGMENT_GAP)).whitespace_nowrap().children(segments.clone()).into_any_element()
                        })
                    }))
                    .when_some(self.live_tool.filter(|_| !done), |d, tool| {
                        d.child(Spinner::new(child("spin")).size(px(12.)).color(muted.opacity(0.7))).child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .overflow_hidden()
                                .text_color(theme.foreground.opacity(0.75))
                                .child(Morph::new(child("live"), tool.clone(), text(tool))),
                        )
                    }),
            ));

        div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(radius::xxl())
            .bg(theme.card)
            .child(header)
            .when(has_calls && reveal > 0.001, |d| {
                d.child(
                    div()
                        .relative()
                        .top(px(-4. * (1. - reveal)))
                        .opacity(reveal)
                        .flex()
                        .flex_col()
                        .px(px(14.))
                        .pb(px(8.))
                        .children(self.calls),
                )
            })
    }
}
