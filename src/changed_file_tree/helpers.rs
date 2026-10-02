use super::{FoldRow, OpenRow, SelectDown, SelectUp, UnfoldRow};

use std::collections::HashSet;

use gpui_kit::{
    App, IntoElement, KeyBinding, ParentElement, SharedString, Styled, div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{file_tree::TreeRow, theme::Theme, typography::MONO_FONT_FAMILY};
use super::types::{CONTEXT, TreeKey};

pub(crate) fn bind_keys(cx: &mut App) {
    let context = Some(CONTEXT);
    cx.bind_keys([
        KeyBinding::new("up", SelectUp, context),
        KeyBinding::new("down", SelectDown, context),
        KeyBinding::new("left", FoldRow, context),
        KeyBinding::new("right", UnfoldRow, context),
        KeyBinding::new("enter", OpenRow, context),
    ]);
}

/// Whether opening `folder` opens the folded folder `folded`: it is `folder` itself or one of its ancestors.
pub fn unfolds(folded: &str, folder: &str) -> bool {
    folded == folder || folder.strip_prefix(folded).is_some_and(|rest| rest.starts_with('/'))
}

/// What a key does to the keyboard row and the folds, given the rows on screen. Pure, so it is tested
/// without a window. Returns the path to open, if the key opens a file.
pub(crate) fn key(
    rows: &[TreeRow],
    cursor: &mut Option<SharedString>,
    folded: &mut HashSet<SharedString>,
    action: TreeKey,
) -> Option<SharedString> {
    let at = cursor.as_ref().and_then(|c| rows.iter().position(|r| &r.path == c));
    let row = at.map(|i| &rows[i]);
    let go = |cursor: &mut Option<SharedString>, i: usize| *cursor = rows.get(i).map(|r| r.path.clone());
    match action {
        TreeKey::Up => go(cursor, at.map_or(rows.len().saturating_sub(1), |i| i.saturating_sub(1))),
        TreeKey::Down => go(cursor, at.map_or(0, |i| (i + 1).min(rows.len().saturating_sub(1)))),
        TreeKey::Left => match row {
            Some(r) if r.is_folder() && !r.folded => {
                folded.insert(r.path.clone());
            }
            Some(r) => {
                // The nearest row above at a shallower depth is the parent.
                if let Some(parent) = rows[..at.unwrap_or(0)].iter().rev().find(|p| p.depth < r.depth) {
                    *cursor = Some(parent.path.clone());
                }
            }
            None => go(cursor, 0),
        },
        TreeKey::Right => match row {
            Some(r) if r.is_folder() && r.folded => {
                folded.remove(&r.path);
            }
            Some(r) if r.is_folder() => go(cursor, at.unwrap_or(0) + 1),
            _ => {}
        },
        TreeKey::Enter => match row {
            Some(r) if r.is_folder() => {
                if !folded.remove(&r.path) {
                    folded.insert(r.path.clone());
                }
            }
            Some(r) => return Some(r.path.clone()),
            None => {}
        },
    }
    None
}

pub(super) fn counts(id: impl Into<gpui_kit::ElementId>, added: usize, removed: usize, theme: &Theme) -> impl IntoElement {
    let id = id.into();
    let count = |text: String, added: bool| {
        div().font_family(MONO_FONT_FAMILY).text_size(px(11.)).text_color(theme.diff_color(added).opacity(0.9)).child(crate::Digits::new((id.clone(), if added { "added" } else { "removed" }), text, px(11.)))
    };
    div()
        .flex()
        .flex_none()
        .gap(px(4.))
        .when(added > 0, |d| d.child(count(format!("+{added}"), true)))
        .when(removed > 0, |d| d.child(count(format!("\u{2212}{removed}"), false)))
}
