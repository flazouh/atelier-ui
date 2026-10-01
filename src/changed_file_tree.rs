//! The changed files as a tree, left of the diff, for the agent's turn and the pull request view alike.
//!
//! Folders show their summed `+a −r` and fold; a folder that holds only one folder merges with it into
//! one row ("crates/beui/src"). Files show a file icon, the name, `+a −r`, and a check once reviewed.
//! The current file has the accent's selection wash. Pressing a file reports it; pressing a folder folds
//! it.
//!
//! Keyboard, while the tree has focus: up and down move the keyboard row, left folds a folder or goes to
//! its parent, right opens a folder or goes to its first child, enter opens a file or folds a folder.
//! Bare `j` and `k` move between files through the owner's [`crate::review::ReviewHandlers`]. Folding
//! is instant: it happens tens of times a session, and the rows it hides are one keystroke away.

use std::{collections::HashSet, rc::Rc, sync::Arc};

use gpui_kit::{
    App, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement, KeyBinding, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    changed_files::ChangedFile,
    file_icon::FileIcon,
    file_tree::{FileTree, TreeRow},
    icon::{Icon, IconName},
    theme::{ActiveTheme, Theme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

gpui_kit::actions!(
    changed_file_tree,
    [
        /// Moves the keyboard row up.
        SelectUp,
        /// Moves the keyboard row down.
        SelectDown,
        /// Folds the folder, or goes to the parent folder.
        FoldRow,
        /// Opens the folder, or goes to its first child.
        UnfoldRow,
        /// Opens the file, or folds and opens the folder.
        OpenRow,
    ]
);

const CONTEXT: &str = "ChangedFileTree";
/// How far each level steps in.
const INDENT: f32 = 12.;

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

type OpenHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct ChangedFileTree {
    id: ElementId,
    files: Vec<ChangedFile>,
    reviewed: HashSet<SharedString>,
    current: Option<SharedString>,
    on_open: Option<OpenHandler>,
    focus: Option<FocusHandle>,
    reveal: Option<(u64, SharedString)>,
}

impl ChangedFileTree {
    pub fn new(id: impl Into<ElementId>, files: Vec<ChangedFile>) -> Self {
        Self { id: id.into(), files, reviewed: HashSet::new(), current: None, on_open: None, focus: None, reveal: None }
    }

    /// Opens `folder` and the folders round it. It does this once for each `token`: the owner bumps the token to ask again, as
    /// a press on a breadcrumb does.
    pub fn reveal(mut self, token: u64, folder: impl Into<SharedString>) -> Self {
        self.reveal = Some((token, folder.into()));
        self
    }

    /// The files that carry a check.
    pub fn reviewed(mut self, reviewed: HashSet<SharedString>) -> Self {
        self.reviewed = reviewed;
        self
    }

    /// The open file, which has the selection wash.
    pub fn current(mut self, path: impl Into<SharedString>) -> Self {
        self.current = Some(path.into());
        self
    }

    /// The owner's focus handle, so it can move focus to the tree, as a key to jump there would. Without
    /// one the tree keeps its own.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    pub fn on_open(mut self, handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }
}

/// Whether opening `folder` opens the folded folder `folded`: it is `folder` itself or one of its ancestors.
pub fn unfolds(folded: &str, folder: &str) -> bool {
    folded == folder || folder.strip_prefix(folded).is_some_and(|rest| rest.starts_with('/'))
}

/// What the tree keeps between frames: its focus, its folds, and the keyboard row.
struct TreeState {
    focus: FocusHandle,
    folded: HashSet<SharedString>,
    cursor: Option<SharedString>,
    revealed: u64,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TreeKey {
    Up,
    Down,
    Left,
    Right,
    Enter,
}

fn counts(id: impl Into<gpui_kit::ElementId>, added: usize, removed: usize, theme: &Theme) -> impl IntoElement {
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

impl RenderOnce for ChangedFileTree {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| TreeState {
            focus: cx.focus_handle(),
            folded: HashSet::new(),
            cursor: None,
            revealed: 0,
        });
        if let Some((token, folder)) = &self.reveal {
            let asked = *token != state.read(cx).revealed;
            if asked {
                state.update(cx, |s, _| {
                    s.revealed = *token;
                    let folder = folder.as_ref();
                    s.folded.retain(|f| !unfolds(f.as_ref(), folder));
                });
            }
        }
        let tree = FileTree::new(&self.files);
        let (focus, rows, cursor) = {
            let s = state.read(cx);
            (self.focus.clone().unwrap_or_else(|| s.focus.clone()), tree.rows(&s.folded), s.cursor.clone())
        };
        let focused = focus.is_focused(window);
        let child = |name: String| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());

        let on_key = |action: TreeKey| {
            let (state, tree, open) = (state.clone(), tree.clone(), self.on_open.clone());
            move |window: &mut Window, cx: &mut App| {
                let opened = state.update(cx, |s, cx| {
                    let rows = tree.rows(&s.folded);
                    let opened = key(&rows, &mut s.cursor, &mut s.folded, action);
                    cx.notify();
                    opened
                });
                if let (Some(path), Some(open)) = (opened, &open) {
                    open(&path, window, cx);
                }
            }
        };
        let (up, down, left, right, enter) =
            (on_key(TreeKey::Up), on_key(TreeKey::Down), on_key(TreeKey::Left), on_key(TreeKey::Right), on_key(TreeKey::Enter));

        let row_el = |row: TreeRow| {
            let is_current = self.current.as_ref() == Some(&row.path);
            let is_cursor = focused && cursor.as_ref() == Some(&row.path);
            let reviewed = !row.is_folder() && self.reviewed.contains(&row.path);
            let (state, open, path, is_folder) = (state.clone(), self.on_open.clone(), row.path.clone(), row.is_folder());
            let focus = focus.clone();
            div()
                .id(child(format!("row-{}", row.path)))
                .relative()
                .flex()
                .items_center()
                .gap(px(6.))
                .h(px(28.))
                .pl(px(8. + INDENT * row.depth as f32))
                .pr(px(8.))
                .rounded(radius::md())
                .cursor_pointer()
                .text_size(TextSize::Xs.font_size())
                .when(is_current, |d| d.bg(theme.accent.opacity(0.18)))
                .when(!is_current && is_cursor, |d| d.bg(theme.muted_hover()))
                .when(!is_current, |d| d.hover(|s| s.bg(theme.muted_hover())))
                .when(is_cursor, |d| d.child(crate::focus::row_ring(&theme, theme.background, radius::md())))
                .on_click(move |_, window, cx| {
                    focus.focus(window, cx);
                    state.update(cx, |s, cx| {
                        s.cursor = Some(path.clone());
                        if is_folder && !s.folded.remove(&path) {
                            s.folded.insert(path.clone());
                        }
                        cx.notify();
                    });
                    if let (false, Some(open)) = (is_folder, &open) {
                        open(&path, window, cx);
                    }
                })
                // A folder: its fold mark, then its folder, open while unfolded. A file: the fold mark's
                // room, so names line up, then its own icon.
                .child(if row.is_folder() {
                    Icon::new(if row.folded { IconName::ChevronRight } else { IconName::ChevronDown })
                        .size(px(12.))
                        .color(muted.opacity(0.7))
                        .into_any_element()
                } else {
                    div().flex_none().w(px(12.)).into_any_element()
                })
                .child(if row.is_folder() { FileIcon::folder(&row.name, !row.folded) } else { FileIcon::file(&row.path) })
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .when(row.is_folder(), |d| d.text_color(muted))
                        .when(!row.is_folder(), |d| {
                            d.text_color(theme.foreground.opacity(0.9)).when(is_current, |d| d.font_weight(FontWeight::MEDIUM))
                        })
                        .child(row.name.clone()),
                )
                .child(counts(gpui_kit::SharedString::from(format!("counts-{}", row.path)), row.added, row.removed, &theme))
                .child(
                    div()
                        .flex_none()
                        .w(px(12.))
                        .when(reviewed, |d| d.child(Icon::new(IconName::Check).size(px(12.)).color(theme.success))),
                )
        };

        div()
            .id(self.id.clone())
            .key_context(CONTEXT)
            .track_focus(&focus)
            .on_action(move |_: &SelectUp, window, cx| up(window, cx))
            .on_action(move |_: &SelectDown, window, cx| down(window, cx))
            .on_action(move |_: &FoldRow, window, cx| left(window, cx))
            .on_action(move |_: &UnfoldRow, window, cx| right(window, cx))
            .on_action(move |_: &OpenRow, window, cx| enter(window, cx))
            .flex()
            .flex_col()
            .gap(px(1.))
            .p(px(6.))
            .children(rows.into_iter().map(row_el))
    }
}

#[cfg(test)]
mod tests;
