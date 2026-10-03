use std::{sync::Arc, time::Instant};

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
    relative,
};

use crate::scale::px;
use crate::{
    focus::PressStop,
    icon::{Icon, IconName},
    motion::{self, Channel, Curve, Spring, ease},
    reveal::Reveal,
    status_mark::{Mark, StatusMark},
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::types::TodoStatus;
use super::helpers::{progress, status_color, title_color, update};

#[derive(Clone, Debug, PartialEq)]
pub struct Todo {
    /// Stable across renders, so a step's motion follows it instead of its row index. Two todos on
    /// screen at once must not share one.
    pub id: SharedString,
    pub text: SharedString,
    pub status: TodoStatus,
    /// Progress in percent for an in-progress step. Without it the arc spins.
    pub progress: Option<f32>,
    /// A short fact on the right, such as "100%".
    pub detail: Option<SharedString>,
}

impl Todo {
    pub fn new(id: impl Into<SharedString>, text: impl Into<SharedString>, status: TodoStatus) -> Self {
        Self { id: id.into(), text: text.into(), status, progress: None, detail: None }
    }

    pub fn progress(mut self, percent: f32) -> Self {
        self.progress = Some(percent);
        self
    }

    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

#[derive(IntoElement)]
pub struct TodoList {
    pub(super) id: ElementId,
    pub(super) title: SharedString,
    pub(super) todos: Vec<Todo>,
}

impl TodoList {
    pub fn new(id: impl Into<ElementId>, todos: Vec<Todo>) -> Self {
        Self { id: id.into(), title: "To-dos".into(), todos }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }
}

/// Per-step marks, animated between states with beui's timings. Kept by [`Todo::id`], not by row
/// index, so inserting or removing a step never retargets the wrong row's motion.
pub(super) struct RowMotion {
    pub(super) id: SharedString,
    pub(super) status: TodoStatus,
    pub(super) progress: Option<f32>,
    pub(super) fill: Channel,
    arc: Channel,
    arc_alpha: Channel,
    pub(super) check: Channel,
    cross: Channel,
    pub(super) strike: Channel,
}

impl RowMotion {
    pub(super) fn new(id: SharedString, status: TodoStatus, progress: Option<f32>) -> Self {
        let mut row = Self {
            id,
            status,
            progress,
            fill: Channel::new(0.),
            arc: Channel::new(0.),
            arc_alpha: Channel::new(0.),
            check: Channel::new(0.),
            cross: Channel::new(0.),
            strike: Channel::new(0.),
        };
        row.retarget(status, progress, true);
        row
    }

    pub(super) fn retarget(&mut self, status: TodoStatus, progress: Option<f32>, jump: bool) {
        self.status = status;
        self.progress = progress;
        let on = |yes: bool| if yes { 1. } else { 0. };
        let fade = Curve::Ease(0.18, ease::OUT);
        let draw = Curve::Ease(0.24, ease::OUT);
        let layout = Curve::Spring(Spring::LAYOUT);
        let busy = status == TodoStatus::InProgress;
        let arc = if busy { progress.map_or(0.68, |p| p.clamp(0., 100.) / 100.) } else { 0. };
        self.fill.animate(on(status == TodoStatus::Done) * 0.06, fade, 0., jump);
        self.arc.animate(arc, layout, 0., jump);
        self.arc_alpha.animate(on(busy), layout, 0., jump);
        self.check.animate(on(status == TodoStatus::Done), draw, 0., jump);
        self.cross.animate(on(status == TodoStatus::Cancelled), Curve::Ease(0.2, ease::OUT), 0., jump);
        self.strike.animate(on(status == TodoStatus::Done), Curve::Ease(0.28, ease::OUT), 0.06, jump);
    }

    fn channels(&self) -> [&Channel; 6] {
        [&self.fill, &self.arc, &self.arc_alpha, &self.check, &self.cross, &self.strike]
    }
}

pub(super) struct ListMotion {
    pub(super) disclosure: Reveal,
    pub(super) all_done: bool,
    pub(super) header_done: Channel,
    pub(super) rows: Vec<RowMotion>,
    born: Instant,
}

impl RenderOnce for TodoList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let (done, total) = progress(&self.todos);
        let all_done = total > 0 && done == total;
        let motion = window.use_keyed_state(self.id.clone(), cx, move |_, _| ListMotion {
            disclosure: Reveal::new(!all_done),
            all_done,
            header_done: Channel::new(if all_done { 1. } else { 0. }),
            rows: Vec::new(),
            born: Instant::now(),
        });
        update(&motion, &self.todos, reduce, cx);

        let m = motion.read(cx);
        let spinning = self.todos.iter().any(|t| t.status == TodoStatus::InProgress && t.progress.is_none());
        let moving = m.disclosure.is_moving()
            || m.header_done.is_running()
            || m.rows.iter().any(|r| r.channels().iter().any(|c| c.is_running()));
        if moving || (spinning && !reduce) {
            window.request_animation_frame();
        }
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let (reveal, chevron, header_done) =
            (m.disclosure.reveal.value(), m.disclosure.chevron.value(), m.header_done.value());
        let height = m.disclosure.height.clone();
        // One turn per beui's arc duration, linear, like its spinning arc.
        let spin = (m.born.elapsed().as_secs_f32() / motion::duration::TODO_ARC_SPIN.as_secs_f32()).fract();
        let rows: Vec<_> = m.rows.iter().map(|r| r.channels().map(|c| c.value())).collect();

        let icon = div()
            .relative()
            .flex()
            .flex_none()
            .size(px(24.))
            .items_center()
            .justify_center()
            .child(
                div()
                    .absolute()
                    .opacity(1. - header_done)
                    .text_color(muted)
                    .child(Icon::new(IconName::Checklist).size(px(16.))),
            )
            .child(div().absolute().opacity(header_done).child(StatusMark::new(
                Mark {
                    color: theme.success,
                    ring_alpha: 0.,
                    dashed: false,
                    fill_alpha: 1.,
                    arc: 0.,
                    arc_start: 0.,
                    check: 0.,
                    cross: 0.,
                    slash: 0.,
                    glyph: None,
                },
                px(22.),
            )))
            .when(header_done > 0.01, |d| {
                d.child(div().absolute().opacity(header_done).child(StatusMark::new(
                    Mark {
                        color: theme.background,
                        ring_alpha: 0.,
                        dashed: false,
                        fill_alpha: 0.,
                        arc: 0.,
                        arc_start: 0.,
                        check: header_done,
                        cross: 0.,
                        slash: 0.,
                        glyph: None,
                    },
                    px(22.),
                )))
            });

        let toggle = motion.clone();
        let header = div()
            .id(ElementId::NamedChild(Arc::new(self.id.clone()), "header".into()))
            .group("todo-header")
            .flex()
            .items_center()
            .gap(px(10.))
            .h(px(36.))
            .px(px(12.))
            .rounded(radius::card())
            .cursor_pointer()
            .press_stop((self.id.clone(), "head-focus"), crate::theme::radius::md(), window, cx)
            .on_click(move |_, _, cx| {
                let reduce = cx.reduce_motion();
                toggle.update(cx, |m, cx| {
                    let open = !m.disclosure.open;
                    m.disclosure.set_open(open, reduce);
                    cx.notify();
                })
            })
            .child(icon)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(TextSize::Sm.font_size())
                    .line_height(TextSize::Sm.line_height())
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground.opacity(0.9))
                    .child(self.title),
            )
            .child(
                div()
                    .flex_none()
                    .font_family(MONO_FONT_FAMILY)
                    .text_size(TextSize::Xs.font_size())
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if all_done { theme.success } else { muted })
                    .child(format!("{done}/{total}")),
            )
            .child(
                div()
                    .flex_none()
                    .text_color(theme.faint())
                    .group_hover("todo-header", |s| s.text_color(muted))
                    .child(Icon::new(IconName::ChevronDown).size(px(14.)).turn(chevron / 360.)),
            );

        let list = div().flex().flex_col().px(px(8.)).pb(px(8.)).children(self.todos.into_iter().zip(rows).map(
            |(todo, [fill, arc, arc_alpha, check, cross, strike])| {
                let color = status_color(todo.status, &theme);
                let in_progress = todo.status == TodoStatus::InProgress;
                let mark = Mark {
                    color,
                    // beui draws the in-progress track at 20%.
                    ring_alpha: if in_progress { 0.2 } else { 1. },
                    dashed: todo.status == TodoStatus::Pending,
                    fill_alpha: fill,
                    arc: arc * arc_alpha,
                    arc_start: if in_progress && todo.progress.is_none() && !reduce { spin } else { 0. },
                    check,
                    cross,
                    slash: 0.,
                    glyph: None,
                };
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .min_h(px(36.))
                    .px(px(6.))
                    .py(px(4.))
                    .rounded(radius::xl())
                    .child(div().mx(px(2.)).child(StatusMark::new(mark, px(20.))))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(TextSize::Sm.font_size())
                            .line_height(px(20.))
                            .text_color(title_color(todo.status, &theme))
                            .flex()
                            .child(
                                // `relative inline-block max-w-full`: as wide as the text, so the strike stops with it.
                                div()
                                    .relative()
                                    .flex_none()
                                    .max_w_full()
                                    .truncate()
                                    .child(todo.text)
                                    .child(
                                        div()
                                            .absolute()
                                            .left_0()
                                            .top(relative(0.5))
                                            .h(px(1.))
                                            .w(relative(strike))
                                            .opacity(strike)
                                            .bg(title_color(todo.status, &theme)),
                                    ),
                            ),
                    )
                    .when_some(todo.detail, |d, detail| {
                        d.child(
                            div()
                                .flex_none()
                                .text_size(TextSize::Sm.font_size())
                                .text_color(muted)
                                .child(detail),
                        )
                    })
            },
        ));

        div()
            .flex()
            .flex_col()
            .w_full()
            .overflow_hidden()
            .rounded(radius::card())
            .bg(theme.card_strong)
            .child(header)
            .when(reveal > 0.001, |d| {
                d.child(crate::reveal::body(list, reveal, &height))
            })
    }
}
