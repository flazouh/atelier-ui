//! A project's worktrees, for the Git view: the main checkout first, then each worktree with its branch, its folder
//! and what it holds in plain words ("3 uncommitted", "2 only here", "merged"), coloured by what they mean for
//! removing it. It shows; it removes nothing.
use gpui_kit::{
    App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::FluentBuilder,
};

use crate::{
    icon::{Icon, IconName},
    scale::px,
    theme::{ActiveTheme, Theme},
    typography::TextSize,
};

/// What a note means for removing the worktree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteTone {
    /// Worth knowing, no risk: "behind 2".
    Quiet,
    /// Removing it would lose this: "3 uncommitted", "2 only here".
    Warning,
    /// Safe to remove for this reason: "merged".
    Good,
}

/// The colour of a note.
pub fn note_colour(tone: NoteTone, theme: &Theme) -> Hsla {
    match tone {
        NoteTone::Quiet => theme.muted_foreground,
        NoteTone::Warning => theme.warning,
        NoteTone::Good => theme.success,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeNote {
    pub words: SharedString,
    pub tone: NoteTone,
}

impl WorktreeNote {
    pub fn new(words: impl Into<SharedString>, tone: NoteTone) -> Self {
        Self {
            words: words.into(),
            tone,
        }
    }
}

/// One worktree, as the list shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeRow {
    /// Its folder on the host, which names the row.
    pub path: SharedString,
    /// Its folder as shown, such as `~/code/atelier-fix`.
    pub folder: SharedString,
    /// `None` when its HEAD is detached.
    pub branch: Option<SharedString>,
    pub main: bool,
    pub notes: Vec<WorktreeNote>,
    /// The sessions working in it.
    pub sessions: usize,
}

#[derive(IntoElement)]
pub struct WorktreeList {
    id: ElementId,
    rows: Vec<WorktreeRow>,
}

impl WorktreeList {
    pub fn new(id: impl Into<ElementId>, rows: Vec<WorktreeRow>) -> Self {
        Self {
            id: id.into(),
            rows,
        }
    }
}

fn sessions_words(n: usize) -> Option<String> {
    match n {
        0 => None,
        1 => Some("1 session".into()),
        n => Some(format!("{n} sessions")),
    }
}

impl RenderOnce for WorktreeList {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let count = self.rows.len();
        let rows = self.rows.into_iter().map(|row| {
            let selector: SharedString = format!("worktree-{}", row.path).into();
            let badge: SharedString = format!("worktree-main-{}", row.path).into();
            let detached = row.branch.is_none();
            let mut facts: Vec<(SharedString, Hsla)> = row
                .notes
                .iter()
                .map(|n| (n.words.clone(), note_colour(n.tone, &theme)))
                .collect();
            facts.extend(sessions_words(row.sessions).map(|w| (w.into(), muted)));
            div()
                .id(ElementId::Name(selector.clone()))
                .debug_selector(move || selector.to_string())
                .flex()
                .flex_col()
                .flex_none()
                .gap(px(2.))
                .mx(px(4.))
                .px(px(8.))
                .py(px(6.))
                .text_size(TextSize::Sm.font_size())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(Icon::new(IconName::PrOpen).size(px(14.)).color(muted))
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .when(detached, |d| d.text_color(muted))
                                .child(row.branch.unwrap_or_else(|| "detached".into())),
                        )
                        .when(row.main, |d| {
                            d.child(
                                div()
                                    .debug_selector(move || badge.to_string())
                                    .flex_none()
                                    .text_size(TextSize::Xs.font_size())
                                    .text_color(muted)
                                    .child("main checkout"),
                            )
                        }),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .pl(px(20.))
                        .text_size(TextSize::Xs.font_size())
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .text_color(muted)
                                .child(row.folder),
                        )
                        .children(facts.into_iter().map(|(words, colour)| {
                            div().flex_none().text_color(colour).child(words)
                        })),
                )
        });
        div()
            .id(self.id)
            .debug_selector(|| "worktrees".into())
            .flex()
            .flex_col()
            .child(
                div()
                    .px(px(12.))
                    .pt(px(12.))
                    .pb(px(4.))
                    .text_size(TextSize::Xs.font_size())
                    .text_color(muted)
                    .child(format!("Worktrees · {count}")),
            )
            .children(rows)
    }
}

#[cfg(test)]
pub(crate) mod tests;
