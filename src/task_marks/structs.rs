use gpui_kit::{App, IntoElement, Pixels, RenderOnce, Styled, Window, canvas};

use crate::scale::px;
use crate::{
    task_model::{Priority, TaskStatus},
    theme::{ActiveTheme},
};
use super::helpers::{paint_priority, paint_status};

/// A task's status as a mark.
#[derive(IntoElement)]
pub struct TaskStatusMark {
    pub(super) status: TaskStatus,
    pub(super) size: Pixels,
}

impl TaskStatusMark {
    pub fn new(status: TaskStatus) -> Self {
        Self { status, size: px(16.) }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for TaskStatusMark {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let (status, theme) = (self.status, cx.theme().clone());
        canvas(|_, _, _| {}, move |bounds, _, window, _| paint_status(bounds, status, &theme, window)).size(self.size).flex_none()
    }
}

/// A task's priority as a mark.
#[derive(IntoElement)]
pub struct PriorityMark {
    pub(super) priority: Priority,
    pub(super) size: Pixels,
}

impl PriorityMark {
    pub fn new(priority: Priority) -> Self {
        Self { priority, size: px(16.) }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for PriorityMark {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let (priority, theme) = (self.priority, cx.theme().clone());
        canvas(|_, _, _| {}, move |bounds, _, window, _| paint_priority(bounds, priority, &theme, window)).size(self.size).flex_none()
    }
}
