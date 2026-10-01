use gpui_kit::{IntoElement, ParentElement, SharedString, Styled, div, prelude::FluentBuilder};

use crate::scale::px;
use crate::{
    theme::{Theme},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::structs::{ChangedFile, Fold};
use super::types::FOLD_AT;

pub fn header_text(files: usize) -> SharedString {
    if files == 1 { "1 file changed".into() } else { format!("{files} files changed").into() }
}

/// Lines added and removed across every file.
pub fn totals(files: &[ChangedFile]) -> (usize, usize) {
    files.iter().fold((0, 0), |(a, r), f| (a + f.added, r + f.removed))
}

/// Five rows show until the list is expanded. A list only folds when at least two rows would hide.
pub fn fold(total: usize, expanded: bool) -> Fold {
    if expanded || total <= FOLD_AT + 1 {
        Fold { shown: total, hidden: 0 }
    } else {
        Fold { shown: FOLD_AT, hidden: total - FOLD_AT }
    }
}

/// The fold button's words, or `None` when the list is too short to fold.
pub fn fold_label(total: usize, expanded: bool) -> Option<SharedString> {
    let folded = fold(total, false).hidden;
    match (folded, expanded) {
        (0, _) => None,
        (_, true) => Some("Show fewer".into()),
        (n, false) => Some(format!("Show {n} more").into()),
    }
}

/// The folder, with its trailing `/`, and the file name.
pub fn split_path(path: &str) -> (&str, &str) {
    path.rfind('/').map_or(("", path), |i| path.split_at(i + 1))
}

pub fn first_path(files: &[ChangedFile]) -> Option<&SharedString> {
    files.first().map(|f| &f.path)
}

/// `+a` and `−r` in the diff colours, each only when it is not zero.
pub(super) fn counts(id: impl Into<gpui_kit::ElementId>, added: usize, removed: usize, theme: &Theme) -> impl IntoElement {
    let id = id.into();
    let count = |text: String, added: bool| {
        div().font_family(MONO_FONT_FAMILY).text_size(TextSize::Xs.font_size()).text_color(theme.diff_color(added)).child(crate::Digits::new((id.clone(), if added { "added" } else { "removed" }), text, TextSize::Xs.font_size()))
    };
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(6.))
        .when(added > 0, |d| d.child(count(format!("+{added}"), true)))
        .when(removed > 0, |d| d.child(count(format!("\u{2212}{removed}"), false)))
}
