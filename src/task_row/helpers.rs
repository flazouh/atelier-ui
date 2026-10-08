use gpui_kit::{AnyElement, ElementId, IntoElement, ParentElement, Styled, div};

use super::types::MAX_LABELS;
use crate::scale::px;
use crate::{
    task_marks::label_tone_color,
    task_model::{Assignee, Label},
    theme::Theme,
};

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
        .child(
            div()
                .size(px(6.))
                .rounded_full()
                .bg(label_tone_color(label, theme)),
        )
        .child(label.name.clone())
        .into_any_element()
}

/// The assignee's mark: a person's initial in a circle, or the agent's own mark.
pub fn assignee_mark(
    id: impl Into<ElementId>,
    assignee: &Assignee,
    size: f32,
    theme: &Theme,
) -> AnyElement {
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
            .child(
                look.mark
                    .sprite(id, look.mark.working)
                    .size(px(size * 0.7))
                    .playing(false),
            )
            .into_any_element(),
    }
}
