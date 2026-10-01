use std::{rc::Rc, sync::Arc};

use gpui_kit::{
    App, ElementId, FontWeight, HighlightStyle, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, StyledText, Window, div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonVariant},
    entrance::EntranceList,
    file_icon::FileIcon,
    icon::{Icon, IconName},
    focus::PressStop,
    morph::Morph,
    reveal::Reveal,
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::{FileChange, PathHandler};
use super::helpers::{counts, first_path, fold, fold_label, header_text, split_path, totals};

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

/// How many rows show and how many fold away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fold {
    pub shown: usize,
    pub hidden: usize,
}

#[derive(IntoElement)]
pub struct ChangedFiles {
    pub(super) id: ElementId,
    pub(super) files: Vec<ChangedFile>,
    running: bool,
    default_open: bool,
    collapsible: bool,
    on_open_file: Option<PathHandler>,
    on_review: Option<PathHandler>,
}

impl ChangedFiles {
    pub fn new(id: impl Into<ElementId>, files: Vec<ChangedFile>) -> Self {
        Self { id: id.into(), files, running: false, default_open: false, collapsible: false, on_open_file: None, on_review: None }
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

    /// Starts as the header alone, which opens and folds the rows.
    pub fn collapsible(mut self) -> Self {
        self.collapsible = true;
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
        let collapsible = self.collapsible;
        let body = window.use_keyed_state((self.id.clone(), "body"), cx, move |_, _| Reveal::new(!collapsible));
        if body.read(cx).is_moving() {
            window.request_animation_frame();
        }
        let unfolded = body.read(cx).reveal.value();
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
        let chevron = collapsible.then(|| {
            Icon::new(IconName::ChevronRight).size(px(16.)).color(muted).turn(0.25 * unfolded)
        });
        let toggle = body.clone();
        let header = div()
            .id(child("toggle"))
            .debug_selector(|| "changed-files-toggle".into())
            .flex()
            .items_center()
            .gap(px(10.))
            .h(px(if collapsible { 36. } else { 44. }))
            .px(px(if collapsible { 10. } else { 14. }))
            .when(collapsible, |d| {
                d.cursor_pointer().on_click(move |_, _, cx| {
                    let reduce = cx.reduce_motion();
                    toggle.update(cx, |b, cx| {
                        let open = !b.open;
                        b.set_open(open, reduce);
                        cx.notify();
                    })
                })
            })
            .children(chevron)
            .child(div().flex_1().min_w_0().child(Morph::new(child("heading"), summary, heading)))
            .child(
                Button::new(child("review"))
                    .debug_name("changed-files-review")
                    .label("Review")
                    .variant(ButtonVariant::Primary)
                    .disabled(self.running || review.is_none())
                    .when_some(review, |b, (path, handler)| {
                        b.on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            handler(&path, window, cx)
                        })
                    }),
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
            let selector = format!("changed-file-{}", file.path);
            div()
                .id(child(&format!("file-{}", file.path)))
                .debug_selector(move || selector)
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
            .rounded(radius::card())
            .bg(theme.card)
            .child(header)
            .when(unfolded > 0.001, |d| d.child(
                div()
                    .flex()
                    .flex_col()
                    .px(px(8.))
                    .pb(px(8.))
                    .when(collapsible, |d| d.opacity(unfolded))
                    .child(visible)
                    .when(has_rest && reveal > 0.001, |d| {
                        // Its own list: its first paint is the reveal, so only later files enter.
                        d.child(div().relative().top(px(-4. * (1. - reveal))).opacity(reveal).child(entering("rest", rest, window, cx)))
                    })
                    .when_some(fold_button, |d, b| d.child(b)),
            ))

    }
}
