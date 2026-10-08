//! A task as a card on the board: priority, key and assignee on the first line, the title over two lines,
//! then the labels and the count of pull requests. Every card is [`CARD_HEIGHT`] tall, so a column is a
//! virtual list.
use gpui_kit::{
    App, AppContext, Context, ElementId, InteractiveElement, IntoElement, ParentElement, Render, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    icon::{Icon, IconName},
    task_board_model::CARD_HEIGHT,
    task_marks::PriorityMark,
    task_model::TaskData,
    task_row::{assignee_mark, label_chip, shown_labels},
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

/// A card being dragged: what the pointer carries.
#[derive(Clone)]
pub struct DraggedTask {
    pub id: SharedString,
    pub key: SharedString,
    pub title: SharedString,
}

/// What a dragged card shows beside the pointer.
pub struct TaskGhost(pub DraggedTask);

impl Render for TaskGhost {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .w(px(240.))
            .px(px(10.))
            .py(px(8.))
            .rounded(radius::lg())
            .bg(theme.card_strong)
            .text_size(TextSize::Sm.font_size())
            .text_color(theme.foreground)
            .child(div().text_size(TextSize::Xs.font_size()).font_family(MONO_FONT_FAMILY).text_color(theme.muted_foreground).child(self.0.key.clone()))
            .child(self.0.title.clone())
    }
}

#[derive(IntoElement)]
pub struct TaskCard {
    id: ElementId,
    task: TaskData,
    lifted: f32,
    cursor: bool,
}

impl TaskCard {
    pub fn new(id: impl Into<ElementId>, task: TaskData) -> Self {
        Self { id: id.into(), task, lifted: 0., cursor: false }
    }

    /// The keyboard cursor is on this card.
    pub fn cursor(mut self, cursor: bool) -> Self {
        self.cursor = cursor;
        self
    }

    /// The card sits this many pixels above its place, for the spring that settles it after a drop.
    pub fn lifted(mut self, pixels: f32) -> Self {
        self.lifted = pixels;
        self
    }
}

impl RenderOnce for TaskCard {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let task = self.task;
        let (labels, more) = shown_labels(&task.labels);
        let drag = crate::task_card::DraggedTask { id: task.id.clone(), key: task.key.clone(), title: task.title.clone() };
        div()
            .id(self.id.clone())
            .relative()
            .top(px(-self.lifted))
            .flex()
            .flex_col()
            .gap(px(6.))
            .w_full()
            .h(px(CARD_HEIGHT))
            .px(px(10.))
            .py(px(8.))
            .rounded(radius::lg())
            .bg(if self.cursor { theme.card_strong } else { theme.card })
            .border_1()
            .border_color(gpui_kit::transparent_black())
            .when(self.cursor, |d| d.child(crate::focus::row_ring(&theme, theme.background, radius::lg())))
            .cursor_pointer()
            .text_size(TextSize::Sm.font_size())
            .hover(|s| s.bg(theme.card_strong))
            .on_drag(drag, |dragged, _, _, cx| cx.new(|_| TaskGhost(dragged.clone())))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(PriorityMark::new(task.priority))
                    .child(div().font_family(MONO_FONT_FAMILY).text_size(TextSize::Xs.font_size()).text_color(muted).child(task.key.clone()))
                    .child(div().flex_1())
                    .when(!task.prs.is_empty(), |d| {
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(3.))
                                .text_size(TextSize::Xs.font_size())
                                .text_color(muted)
                                .child(Icon::new(IconName::PrMerged).size(px(12.)))
                                .child(task.prs.len().to_string()),
                        )
                    })
                    .children(task.assignee.as_ref().map(|a| assignee_mark((self.id.clone(), "assignee"), a, 18., &theme))),
            )
            .child(div().flex_1().min_h_0().overflow_hidden().line_clamp(2).text_color(theme.foreground).child(task.title))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .children(labels.iter().map(|l| label_chip(l, &theme)))
                    .when(more > 0, |d| d.child(div().text_size(TextSize::Xs.font_size()).text_color(muted).child(format!("+{more}")))),
            )
    }
}
