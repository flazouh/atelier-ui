use super::{FoldRow, OpenRow, SelectDown, SelectUp, UnfoldRow};

use std::{collections::HashSet, rc::Rc, sync::Arc};

use gpui_kit::{
    App, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, uniform_list,
};

use super::helpers::{counts, key, unfolds};
use super::types::{CONTEXT, INDENT, OpenHandler, ROW_GAP, TreeKey};
use crate::scale::px;
use crate::{
    changed_files::ChangedFile,
    file_icon::FileIcon,
    file_tree::{FileTree, TreeRow},
    icon::{Icon, IconName},
    theme::{ActiveTheme, radius},
    typography::TextSize,
};

#[derive(IntoElement)]
pub struct ChangedFileTree {
    pub(super) id: ElementId,
    pub(super) files: Vec<ChangedFile>,
    pub(super) reviewed: HashSet<SharedString>,
    pub(super) current: Option<SharedString>,
    on_open: Option<OpenHandler>,
    pub(super) focus: Option<FocusHandle>,
    reveal: Option<(u64, SharedString)>,
    virtualised: bool,
}

impl ChangedFileTree {
    pub fn new(id: impl Into<ElementId>, files: Vec<ChangedFile>) -> Self {
        Self {
            id: id.into(),
            files,
            reviewed: HashSet::new(),
            current: None,
            on_open: None,
            focus: None,
            reveal: None,
            virtualised: false,
        }
    }

    /// Draws only the rows in view, and scrolls them itself. The tree then fills the height its parent gives it, so
    /// the parent must not scroll it, and a long list costs what a screen of it costs.
    pub fn virtualised(mut self) -> Self {
        self.virtualised = true;
        self
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

    pub fn on_open(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
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
            (
                self.focus.clone().unwrap_or_else(|| s.focus.clone()),
                tree.rows(&s.folded),
                s.cursor.clone(),
            )
        };
        let focused = focus.is_focused(window);

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
        let (up, down, left, right, enter) = (
            on_key(TreeKey::Up),
            on_key(TreeKey::Down),
            on_key(TreeKey::Left),
            on_key(TreeKey::Right),
            on_key(TreeKey::Enter),
        );

        let (row_id, current, reviewed_set, on_open) = (
            self.id.clone(),
            self.current.clone(),
            self.reviewed.clone(),
            self.on_open.clone(),
        );
        let (row_state, row_focus, row_theme) = (state.clone(), focus.clone(), theme.clone());
        let row_el = Rc::new(move |row: TreeRow| {
            let theme = &row_theme;
            let child = |name: String| ElementId::NamedChild(Arc::new(row_id.clone()), name.into());
            let is_current = current.as_ref() == Some(&row.path);
            let is_cursor = focused && cursor.as_ref() == Some(&row.path);
            let reviewed = !row.is_folder() && reviewed_set.contains(&row.path);
            let (state, open, path, is_folder) = (
                row_state.clone(),
                on_open.clone(),
                row.path.clone(),
                row.is_folder(),
            );
            let focus = row_focus.clone();
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
                .when(is_cursor, |d| {
                    d.child(crate::focus::row_ring(
                        theme,
                        theme.background,
                        radius::md(),
                    ))
                })
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
                    Icon::new(if row.folded {
                        IconName::ChevronRight
                    } else {
                        IconName::ChevronDown
                    })
                    .size(px(12.))
                    .color(theme.faint())
                    .into_any_element()
                } else {
                    div().flex_none().w(px(12.)).into_any_element()
                })
                .child(if row.is_folder() {
                    FileIcon::folder(&row.name, !row.folded)
                } else {
                    FileIcon::file(&row.path)
                })
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .when(row.is_folder(), |d| d.text_color(muted))
                        .when(!row.is_folder(), |d| {
                            d.text_color(theme.foreground.opacity(0.9))
                                .when(is_current, |d| d.font_weight(FontWeight::MEDIUM))
                        })
                        .child(row.name.clone()),
                )
                .child(counts(
                    gpui_kit::SharedString::from(format!("counts-{}", row.path)),
                    row.added,
                    row.removed,
                    theme,
                ))
                .child(div().flex_none().w(px(12.)).when(reviewed, |d| {
                    d.child(
                        Icon::new(IconName::Check)
                            .size(px(12.))
                            .color(theme.success),
                    )
                }))
        });

        let body = div()
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
            .p(px(6.));
        if self.virtualised {
            let rows = Rc::new(rows);
            let list = uniform_list(
                ElementId::NamedChild(Arc::new(self.id.clone()), "rows".into()),
                rows.len(),
                move |range, _, _| {
                    range
                        .map(|i| div().pb(px(ROW_GAP)).child(row_el(rows[i].clone())))
                        .collect()
                },
            );
            return body.size_full().child(list.size_full());
        }
        body.gap(px(ROW_GAP))
            .children(rows.into_iter().map(|row| row_el(row)))
    }
}
