use gpui_kit::{IntoElement, ParentElement, SharedString, Styled, div, prelude::FluentBuilder};

use crate::scale::px;
use crate::{
    theme::{Theme},
    typography::TextSize,
};
use super::structs::ThreadSummary;
use super::types::{FACE, FACE_STEP, SHOWN};

/// How many faces show, and the count for the rest.
pub fn faces(people: &[SharedString]) -> (usize, Option<SharedString>) {
    let shown = people.len().min(SHOWN);
    (shown, (people.len() > SHOWN).then(|| format!("+{}", people.len() - SHOWN).into()))
}

/// The header's words: what is open first, then what is resolved, then the Remarks.
pub fn said_so_far(threads: &[ThreadSummary], remarks: usize) -> SharedString {
    let resolved = threads.iter().filter(|t| t.resolved).count();
    said_by_count(threads.len() - resolved, resolved, remarks)
}

/// The header's words from the counts, for a list that shows only a page of a long conversation.
pub fn said_by_count(open: usize, resolved: usize, remarks: usize) -> SharedString {
    let said = if remarks == 1 { "1 remark".to_string() } else { format!("{remarks} remarks") };
    if open + resolved == 0 {
        return if remarks == 0 { "nothing said yet".into() } else { said.into() };
    }
    let threads_say = match (open, resolved) {
        (open, 0) => format!("{open} open"),
        (0, resolved) => format!("all {resolved} resolved"),
        (open, resolved) => format!("{open} open, {resolved} resolved"),
    };
    if remarks == 0 { threads_say.into() } else { format!("{threads_say}, {said}").into() }
}

/// The threads with the open ones first, each group in its own order.
pub fn open_first(threads: Vec<ThreadSummary>) -> Vec<ThreadSummary> {
    let (resolved, open): (Vec<_>, Vec<_>) = threads.into_iter().partition(|t| t.resolved);
    open.into_iter().chain(resolved).collect()
}

/// A round face with the person's initial, at `left` in its row, ringed in the card's fill so the one
/// behind reads as behind.
pub(super) fn face(name: &SharedString, left: f32, theme: &Theme) -> impl IntoElement {
    let initial: String = name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
    div()
        .absolute()
        .left(px(left))
        .top_0()
        .flex()
        .size(px(FACE))
        .items_center()
        .justify_center()
        .rounded_full()
        .border_2()
        .border_color(theme.card)
        .bg(theme.card_strong)
        .text_size(px(10.))
        .font_weight(gpui_kit::FontWeight::MEDIUM)
        .text_color(theme.muted_foreground)
        .child(initial)
}

pub(super) fn faces_el(people: &[SharedString], theme: &Theme) -> impl IntoElement {
    let (shown, more) = faces(people);
    // A box as wide as the overlapped faces, so the words start after the last one.
    let width = FACE + FACE_STEP * shown.saturating_sub(1) as f32;
    div()
        .flex()
        .flex_none()
        .items_center()
        .child(
            div()
                .relative()
                .flex_none()
                .w(px(width))
                .h(px(FACE))
                .children(people.iter().take(shown).enumerate().map(|(i, p)| face(p, FACE_STEP * i as f32, theme))),
        )
        .when_some(more, |d, more| d.child(div().pl(px(4.)).text_size(TextSize::Xs.font_size()).text_color(theme.muted_foreground).child(more)))
}
