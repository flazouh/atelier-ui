//! The card after a turn that lists every file the agent changed, in TodoList's card language:
//!
//! - Header `h-11 px-3.5`: "3 files changed" and the total `+A −R` in the diff colours, then a small
//!   primary Review button. The counts swap with [`Morph`] while the turn still runs.
//! - One row per file: a file icon, the path with its folder muted and its name in ink, a status word for
//!   added, deleted and renamed files, and `+a −r`. Pressing a row reports that file; Review reports the
//!   first one. Review is off until the turn ends.
//! - A long list shows five rows and "Show 7 more". The rest open with [`Reveal`], and "Show fewer" folds
//!   them away again. A fold never hides a single row.
//! - While the turn runs, each new file enters with [`EntranceList`], above the fold and, once the list is
//!   open, below it too. Rows already there when the list opens come in with the reveal instead.

use std::{rc::Rc, sync::Arc};

use gpui_kit::{
    App, ElementId, FontWeight, HighlightStyle, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, StyledText, Window, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    focus::PressStop,
    button::{Button, ButtonVariant},
    entrance::EntranceList,
    file_icon::FileIcon,
    morph::Morph,
    reveal::Reveal,
    theme::{ActiveTheme, Theme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

/// A callback that receives a file's path.
pub(crate) type PathHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// How many rows show before the rest fold away.
pub const FOLD_AT: usize = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum FileChange {
    #[default]
    Modified,
    Added,
    Deleted,
    Renamed {
        from: SharedString,
    },
}

impl FileChange {
    /// The word after the path. A modified file needs none.
    pub fn word(&self) -> Option<&'static str> {
        match self {
            Self::Modified => None,
            Self::Added => Some("Added"),
            Self::Deleted => Some("Deleted"),
            Self::Renamed { .. } => Some("Renamed"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangedFile {
    pub path: SharedString,
    pub added: usize,
    pub removed: usize,
    pub change: FileChange,
}

impl ChangedFile {
    pub fn new(path: impl Into<SharedString>, added: usize, removed: usize) -> Self {
        Self { path: path.into(), added, removed, change: FileChange::Modified }
    }

    pub fn change(mut self, change: FileChange) -> Self {
        self.change = change;
        self
    }
}

pub fn header_text(files: usize) -> SharedString {
    if files == 1 { "1 file changed".into() } else { format!("{files} files changed").into() }
}

/// Lines added and removed across every file.
pub fn totals(files: &[ChangedFile]) -> (usize, usize) {
    files.iter().fold((0, 0), |(a, r), f| (a + f.added, r + f.removed))
}

/// How many rows show and how many fold away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fold {
    pub shown: usize,
    pub hidden: usize,
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

#[derive(IntoElement)]
pub struct ChangedFiles {
    id: ElementId,
    files: Vec<ChangedFile>,
    running: bool,
    default_open: bool,
    on_open_file: Option<PathHandler>,
    on_review: Option<PathHandler>,
}

impl ChangedFiles {
    pub fn new(id: impl Into<ElementId>, files: Vec<ChangedFile>) -> Self {
        Self { id: id.into(), files, running: false, default_open: false, on_open_file: None, on_review: None }
    }

    /// While the turn runs, new files enter as they arrive and Review waits.
    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    /// Starts with the whole list shown instead of folded.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn on_open_file(mut self, handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_file = Some(Rc::new(handler));
        self
    }

    /// Receives the first file's path.
    pub fn on_review(mut self, handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_review = Some(Rc::new(handler));
        self
    }
}

/// `+a` and `−r` in the diff colours, each only when it is not zero.
fn counts(id: impl Into<gpui_kit::ElementId>, added: usize, removed: usize, theme: &Theme) -> impl IntoElement {
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

impl RenderOnce for ChangedFiles {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let total = self.files.len();
        let open = self.default_open;
        let disclosure = window.use_keyed_state(self.id.clone(), cx, move |_, _| Reveal::new(open));
        if disclosure.read(cx).is_moving() {
            window.request_animation_frame();
        }
        let (expanded, reveal) = {
            let d = disclosure.read(cx);
            (d.open, d.reveal.value())
        };
        let child = |name: &str| ElementId::NamedChild(Arc::new(self.id.clone()), name.to_string().into());

        let (added, removed) = totals(&self.files);
        let summary: SharedString = format!("{} +{added} \u{2212}{removed}", header_text(total)).into();
        let heading = {
            let theme = theme.clone();
            let title = header_text(total);
            move |_: &mut Window, _: &mut App| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .whitespace_nowrap()
                    .child(
                        div()
                            .text_size(TextSize::Sm.font_size())
                            .line_height(TextSize::Sm.line_height())
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground.opacity(0.9))
                            .child(title.clone()),
                    )
                    .child(counts("changed-files-total", added, removed, &theme))
                    .into_any_element()
            }
        };
        let review = first_path(&self.files).cloned().zip(self.on_review.clone());
        let header = div()
            .flex()
            .items_center()
            .gap(px(10.))
            .h(px(44.))
            .px(px(14.))
            .child(div().flex_1().min_w_0().child(Morph::new(child("heading"), summary, heading)))
            .child(
                Button::new(child("review"))
                    .label("Review")
                    .variant(ButtonVariant::Primary)
                    .disabled(self.running || review.is_none())
                    .when_some(review, |b, (path, handler)| b.on_click(move |_, window, cx| handler(&path, window, cx))),
            );

        let row = |file: ChangedFile, window: &mut Window, cx: &mut App| {
            // One run of text, so the path truncates as a whole and the folder and name sit flush.
            let folder = split_path(&file.path).0.len();
            let path = StyledText::new(file.path.clone()).with_highlights([(
                folder..file.path.len(),
                HighlightStyle { color: Some(theme.foreground.opacity(0.9)), ..Default::default() },
            )]);
            let word = file.change.word().map(|w| {
                let color = match file.change {
                    FileChange::Added => theme.success,
                    FileChange::Deleted => theme.danger,
                    _ => muted,
                };
                div().flex_none().text_size(px(11.)).font_weight(FontWeight::MEDIUM).text_color(color).child(w)
            });
            let open = self.on_open_file.clone();
            let pressed = file.path.clone();
            div()
                .id(child(&format!("file-{}", file.path)))
                .flex()
                .items_center()
                .gap(px(10.))
                .min_h(px(32.))
                .px(px(6.))
                .rounded(radius::lg())
                .cursor_pointer()
                .hover(|s| s.bg(theme.muted_hover()))
                .when_some(open, |d, open| d.press_stop(child(&format!("file-focus-{}", file.path)), radius::lg(), window, cx).on_click(move |_, window, cx| open(&pressed, window, cx)))
                .child(FileIcon::file(&file.path))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(TextSize::Sm.font_size())
                        .line_height(TextSize::Sm.line_height())
                        .text_color(muted)
                        .child(path),
                )
                .when_some(word, |d, w| d.child(w))
                .child(counts(gpui_kit::SharedString::from(format!("counts-{}", file.path)), file.added, file.removed, &theme))
        };

        let Fold { shown, .. } = fold(total, false);
        let entering = |name: &str, files: Vec<ChangedFile>, window: &mut Window, cx: &mut App| {
            files.into_iter().fold(EntranceList::new(child(name), div().flex().flex_col()), |list, file| {
                list.item(SharedString::from(file.path.to_string()), row(file, window, cx))
            })
        };
        let mut files = self.files;
        let rest = files.split_off(shown.min(files.len()));
        let has_rest = !rest.is_empty();
        let visible = entering("rows", files, window, cx);
        let fold_button = fold_label(total, expanded).map(|label| {
            let toggle = disclosure.clone();
            div()
                .id(child("fold"))
                .press_stop(child("fold-focus"), radius::lg(), window, cx)
                .flex()
                .items_center()
                .min_h(px(32.))
                .px(px(6.))
                .rounded(radius::lg())
                .cursor_pointer()
                .text_size(TextSize::Xs.font_size())
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted)
                .hover(|s| s.bg(theme.muted_hover()).text_color(theme.foreground))
                .on_click(move |_, _, cx| {
                    let reduce = cx.reduce_motion();
                    toggle.update(cx, |d, cx| {
                        let open = !d.open;
                        d.set_open(open, reduce);
                        cx.notify();
                    })
                })
                .child(label)
        });

        div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(radius::xxl())
            .bg(theme.card)
            .child(header)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .px(px(8.))
                    .pb(px(8.))
                    .child(visible)
                    .when(has_rest && reveal > 0.001, |d| {
                        // Its own list: its first paint is the reveal, so only later files enter.
                        d.child(div().relative().top(px(-4. * (1. - reveal))).opacity(reveal).child(entering("rest", rest, window, cx)))
                    })
                    .when_some(fold_button, |d, b| d.child(b)),
            )
    }
}

#[cfg(test)]
mod tests;
