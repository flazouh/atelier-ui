use std::rc::Rc;

use gpui_kit::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    pr_chip::PrChip,
    session_row::status_mark,
    sidebar_model::since,
    task_marks::{PriorityMark, TaskStatusMark},
    task_model::TaskData,
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::types::{Handler, ROW_HEIGHT};
use super::helpers::{assignee_mark, label_chip, shown_labels};

#[derive(IntoElement)]
pub struct TaskRow {
    pub(super) id: ElementId,
    pub(super) task: TaskData,
    now: u64,
    cursor: bool,
    selected: bool,
    on_click: Option<Handler>,
}

impl TaskRow {
    /// `now` is the time to count "2m" from, in seconds since the Unix epoch.
    pub fn new(id: impl Into<ElementId>, task: TaskData, now: u64) -> Self {
        Self { id: id.into(), task, now, cursor: false, selected: false, on_click: None }
    }

    /// The keyboard cursor is on this row.
    pub fn cursor(mut self, cursor: bool) -> Self {
        self.cursor = cursor;
        self
    }

    /// The row is in the selection (`x`).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// A press; the handler reads the modifiers to tell a plain open from a select.
    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for TaskRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let task = self.task;
        let muted = theme.muted_foreground;
        let (labels, more) = shown_labels(&task.labels);
        let closed = !task.status.is_open();
        let session = task.sessions.first().map(|s| status_mark((self.id.clone(), "session"), &s.look, &s.status, window, cx));
        let pr = task.prs.first().cloned();
        let assignee = task.assignee.as_ref().map(|a| assignee_mark((self.id.clone(), "assignee"), a, 18., &theme));
        let base: SharedString = task.key.clone();
        div()
            .id(self.id.clone())
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(8.))
            .w_full()
            .h(px(ROW_HEIGHT))
            .px(px(10.))
            .rounded(radius::md())
            .cursor_pointer()
            .text_size(TextSize::Sm.font_size())
            .when(self.selected, |d| d.bg(theme.accent.opacity(0.12)))
            .when(self.cursor && !self.selected, |d| d.bg(theme.card_strong))
            .when(self.cursor && self.selected, |d| d.bg(theme.accent.opacity(0.2)))
            .when(self.cursor, |d| d.child(crate::focus::row_ring(&theme, theme.background, radius::md())))
            .hover(|s| s.bg(theme.card_strong.opacity(0.7)))
            .when_some(self.on_click, |d, click| d.on_click(move |event, window, cx| click(event, window, cx)))
            .child(PriorityMark::new(task.priority))
            .child(div().flex_none().w(px(58.)).font_family(MONO_FONT_FAMILY).text_size(TextSize::Xs.font_size()).text_color(muted).child(base))
            .child(TaskStatusMark::new(task.status))
            .child(div().flex_1().min_w_0().truncate().text_color(if closed { muted } else { theme.foreground }).child(task.title))
            .children(labels.iter().map(|l| label_chip(l, &theme)))
            .when(more > 0, |d| d.child(div().flex_none().text_size(TextSize::Xs.font_size()).text_color(muted).child(format!("+{more}"))))
            .children(session)
            .when_some(pr, |d, pr| d.child(PrChip::new((self.id.clone(), "pr"), pr)))
            .children(assignee)
            .when(task.assignee.is_none(), |d| {
                d.child(
                    Icon::new(IconName::Add)
                        .size(px(14.))
                        .color(muted.opacity(0.0)),
                )
            })
            .child(
                div()
                    .flex_none()
                    .w(px(28.))
                    .text_size(TextSize::Xs.font_size())
                    .text_color(muted)
                    .child(since(self.now, task.updated_at)),
            )
    }
}
