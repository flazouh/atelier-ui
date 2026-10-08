use gpui_kit::{Hsla, IntoElement, ParentElement, SharedString, Styled, div};

use super::structs::MultiOption;
use super::types::{EMPTY, GROUP_PAD, LABEL, LIST_PAD, ROW, TRACKING};
use crate::scale::px;

/// The default filter: every letter of the query, in order, somewhere in the value and the words.
pub fn matches(query: &str, option: &MultiOption) -> bool {
    let needle: Vec<char> = query.trim().to_lowercase().chars().collect();
    if needle.is_empty() {
        return true;
    }
    let haystack = std::iter::once(option.value.as_ref())
        .chain(std::iter::once(option.label.as_ref()))
        .chain(option.keywords.iter().map(|k| k.as_ref()))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let mut at = 0;
    for c in haystack.chars() {
        if Some(&c) == needle.get(at) {
            at += 1;
        }
        if at == needle.len() {
            return true;
        }
    }
    false
}

/// The options that show for `query`, in list order.
pub fn visible<'a>(options: &'a [MultiOption], query: &str) -> Vec<&'a MultiOption> {
    options.iter().filter(|o| matches(query, o)).collect()
}

/// The active option: the one the pointer or the keys last moved to, while the query is the one it was placed
/// under and it can still be chosen; else the first chosen value that can be; else the first that can be chosen.
pub fn active<'a>(
    cursor: Option<&(SharedString, SharedString)>,
    query: &str,
    shown: &[&'a MultiOption],
    values: &[SharedString],
) -> Option<&'a SharedString> {
    let enabled: Vec<&MultiOption> = shown.iter().copied().filter(|o| !o.disabled).collect();
    let live = cursor
        .filter(|(_, at)| at.as_ref() == query)
        .and_then(|(value, _)| enabled.iter().find(|o| &o.value == value));
    live.or_else(|| {
        values
            .first()
            .and_then(|v| enabled.iter().find(|o| &o.value == v))
    })
    .or(enabled.first())
    .map(|o| &o.value)
}

/// The value the keys reach from `from`, wrapping round the options that can be chosen.
pub fn move_active<'a>(
    from: Option<&SharedString>,
    shown: &[&'a MultiOption],
    direction: i32,
) -> Option<&'a SharedString> {
    let enabled: Vec<&MultiOption> = shown.iter().copied().filter(|o| !o.disabled).collect();
    if enabled.is_empty() {
        return None;
    }
    let at = from
        .and_then(|f| enabled.iter().position(|o| &o.value == f))
        .unwrap_or(0) as i32;
    let n = enabled.len() as i32;
    Some(&enabled[((at + direction).rem_euclid(n)) as usize].value)
}

/// The height the list wants for `shown`, before the 256px ceiling.
pub fn content_height(shown: &[&MultiOption]) -> f32 {
    if shown.is_empty() {
        return 2. * LIST_PAD + EMPTY;
    }
    let mut height = 2. * LIST_PAD;
    let mut group: Option<&SharedString> = None;
    for (i, option) in shown.iter().enumerate() {
        if i == 0 || option.group.as_ref() != group {
            group = option.group.as_ref();
            height += 2. * GROUP_PAD + if option.group.is_some() { LABEL } else { 0. };
        }
        height += ROW;
    }
    height
}

pub(super) fn tracked(label: &str, size: f32, color: Hsla) -> impl IntoElement {
    div()
        .flex()
        .gap(px(TRACKING))
        .text_size(px(size))
        .text_color(color)
        .children(
            label
                .to_uppercase()
                .chars()
                .map(|c| div().child(c.to_string()))
                .collect::<Vec<_>>(),
        )
}
