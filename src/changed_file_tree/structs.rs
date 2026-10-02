use super::{FoldRow, OpenRow, SelectDown, SelectUp, UnfoldRow};

use std::{collections::HashSet, rc::Rc, sync::Arc};

use gpui_kit::{
    App, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    changed_files::ChangedFile,
    file_icon::FileIcon,
    file_tree::{FileTree, TreeRow},
    icon::{Icon, IconName},
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::{CONTEXT, INDENT, OpenHandler, TreeKey};
use super::helpers::{counts, key, unfolds};

#[derive(IntoElement)]
pub struct ChangedFileTree {
    pub(super) id: ElementId,
    pub(super) files: Vec<ChangedFile>,
    pub(super) reviewed: HashSet<SharedString>,
    pub(super) current: Option<SharedString>,
    on_open: Option<OpenHandler>,
    pub(super) focus: Option<FocusHandle>,
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

/// What the tree keeps between frames: its focus, its folds, and the keyboard row.
struct TreeState {
    pub(super) focus: FocusHandle,
    pub(super) folded: HashSet<SharedString>,
    pub(super) cursor: Option<SharedString>,
    revealed: u64,
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
