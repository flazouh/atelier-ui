//! One task on one line, in the 28 px Small scale: priority, key, status, title, labels, the session and
//! the pull request working on it, the assignee, and the time since the last change. Dense, and virtual in
//! lists (every row is the same height).
use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    icon::{Icon, IconName},
    pr_chip::PrChip,
    session_row::status_mark,
    sidebar_model::since,
    task_marks::{PriorityMark, TaskStatusMark, label_tone_color},
    task_model::{Assignee, Label, TaskData},
    theme::{ActiveTheme, Theme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

pub const ROW_HEIGHT: f32 = 28.;
/// How many labels a row shows before "+n".
pub const MAX_LABELS: usize = 2;

type Handler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// The labels a row shows, and how many more there are.
pub fn shown_labels(labels: &[Label]) -> (&[Label], usize) {
    let shown = labels.len().min(MAX_LABELS);
    (&labels[..shown], labels.len() - shown)
}

/// A label as a small chip: its tone as a dot, then its name.
pub fn label_chip(label: &Label, theme: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(5.))
        .h(px(18.))
        .px(px(7.))
        .rounded_full()
        .bg(theme.card_strong)
        .text_size(px(11.))
        .text_color(theme.muted_foreground)
        .child(div().size(px(6.)).rounded_full().bg(label_tone_color(label, theme)))
        .child(label.name.clone())
        .into_any_element()
}

/// The assignee's mark: a person's initial in a circle, or the agent's own mark.
pub fn assignee_mark(id: impl Into<ElementId>, assignee: &Assignee, size: f32, theme: &Theme) -> AnyElement {
    match assignee {
        Assignee::Person { .. } => div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(size))
            .rounded_full()
            .bg(theme.card_strong)
            .text_size(px(size * 0.55))
            .text_color(theme.foreground)
            .child(assignee.initial())
            .into_any_element(),
        Assignee::Agent { look, .. } => div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(size))
            .rounded_full()
            .bg(theme.card_strong)
            .child(look.mark.sprite(id, look.mark.working).size(px(size * 0.7)).playing(false))
            .into_any_element(),
    }
}

#[derive(IntoElement)]
pub struct TaskRow {
    id: ElementId,
    task: TaskData,
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
            .when(task.assignee.is_none(), |d| d.child(Icon::new(IconName::Add).size(px(14.)).color(muted.opacity(0.0))))
            .child(div().flex_none().w(px(28.)).text_size(TextSize::Xs.font_size()).text_color(muted.opacity(0.8)).child(since(self.now, task.updated_at)))
    }
}

#[cfg(test)]
mod tests;
