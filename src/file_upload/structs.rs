use std::path::{Path, PathBuf};

use gpui_kit::{
    Context, ElementId, EventEmitter, ExternalPaths, FontWeight, InteractiveElement,
    IntoElement, ParentElement, PathPromptOptions, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    cell_bar::CellBar,
    icon::{Icon, IconName},
    layout_motion::shifted,
    motion::{Channel, Curve, Spring, duration, ease, now},
    theme::ActiveTheme,
    typography::TextSize,
};
use super::types::{
    BAR_TIME, FileUploadEvent, ROW_LEAVE, ROW_RISE, ROW_TIME, SWAP_SHIFT, SWAP_TIME,
    UPLOAD_CELLS, UploadStatus, UploadVariant,
};
use super::helpers::{
    clamp_progress, format_bytes, icon_of, kind_of, round_button, status_mark, take_paths,
};

/// One file in the queue.
#[derive(Clone, Debug, PartialEq)]
pub struct UploadItem {
    pub id: SharedString,
    pub name: SharedString,
    pub size: u64,
    /// The media type, such as `video/quicktime`, when it is known.
    pub mime: Option<SharedString>,
    /// 0 to 100.
    pub progress: f32,
    pub status: UploadStatus,
    pub error: Option<SharedString>,
}

impl UploadItem {
    pub fn new(id: impl Into<SharedString>, name: impl Into<SharedString>, size: u64) -> Self {
        Self { id: id.into(), name: name.into(), size, mime: None, progress: 0., status: UploadStatus::Queued, error: None }
    }

    /// The item for the file at `path`: its name and size, uploading at 0.
    pub fn from_path(path: &Path, index: usize) -> std::io::Result<Self> {
        let size = std::fs::metadata(path)?.len();
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        Ok(Self {
            id: format!("{}-{index}-{name}", crate::motion::now_millis()).into(),
            name: name.into(),
            size,
            mime: None,
            progress: 0.,
            status: UploadStatus::Uploading,
            error: None,
        })
    }

    pub fn mime(mut self, mime: impl Into<SharedString>) -> Self {
        self.mime = Some(mime.into());
        self
    }

    pub fn status(mut self, status: UploadStatus) -> Self {
        self.status = status;
        self
    }

    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress;
        self
    }

    pub fn error(mut self, error: impl Into<SharedString>) -> Self {
        self.error = Some(error.into());
        self
    }
}

/// A row of the queue, and the motion it is in.
pub(super) struct Row {
    pub(super) item: UploadItem,
    enter: Channel,
    /// Set when the file left the queue: the row stays, in its place, until this has run.
    exit: Option<Channel>,
    pub(super) bar: Channel,
    /// The status the mark is swapping from, and how far the swap has run (0 to 1: the old leaves, then the new arrives).
    pub(super) swap: Option<(UploadStatus, Channel)>,
}

pub struct FileUpload {
    pub(super) id: ElementId,
    pub(super) rows: Vec<Row>,
    variant: UploadVariant,
    title: SharedString,
    description: SharedString,
    browse_label: SharedString,
    pub(super) accept: Vec<String>,
    pub(super) multiple: bool,
    pub(super) max_files: Option<usize>,
    disabled: bool,
}

impl EventEmitter<FileUploadEvent> for FileUpload {}

impl FileUpload {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            rows: Vec::new(),
            variant: UploadVariant::Row,
            title: "Drop files here".into(),
            description: "Add files to the upload queue".into(),
            browse_label: "Browse".into(),
            accept: Vec::new(),
            multiple: true,
            max_files: None,
            disabled: false,
        }
    }

    pub fn variant(mut self, variant: UploadVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn set_variant(&mut self, variant: UploadVariant, cx: &mut Context<Self>) {
        self.variant = variant;
        cx.notify();
    }

    pub fn words(mut self, title: impl Into<SharedString>, description: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self.description = description.into();
        self
    }

    pub fn set_words(&mut self, title: impl Into<SharedString>, description: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.title = title.into();
        self.description = description.into();
        cx.notify();
    }

    pub fn browse_label(mut self, label: impl Into<SharedString>) -> Self {
        self.browse_label = label.into();
        self
    }

    /// Extensions the queue takes, without the dot; any file when empty.
    pub fn accept(mut self, extensions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.accept = extensions.into_iter().map(Into::into).collect();
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    pub fn max_files(mut self, max: usize) -> Self {
        self.max_files = Some(max);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The files now in the queue, in order (those that are leaving are not).
    pub fn items(&self) -> Vec<UploadItem> {
        self.rows.iter().filter(|r| r.exit.is_none()).map(|r| r.item.clone()).collect()
    }

    fn live(&self) -> usize {
        self.rows.iter().filter(|r| r.exit.is_none()).count()
    }

    pub(super) fn maxed(&self) -> bool {
        self.max_files.is_some_and(|max| self.live() >= max)
    }

    fn new_row(item: UploadItem, reduce: bool) -> Row {
        let mut enter = Channel::new(if reduce { 1. } else { 0. });
        enter.animate(1., Curve::Ease(ROW_TIME, ease::OUT), 0., reduce);
        let bar = Channel::new(clamp_progress(item.progress, item.status) / 100.);
        Row { item, enter, exit: None, bar, swap: None }
    }

    /// Makes the queue these files. A file that is not in the list leaves, and one that is new comes in; one that is in
    /// both takes its new progress and status, the bar moving to the number and the status mark swapping.
    pub fn set_items(&mut self, items: Vec<UploadItem>, cx: &mut Context<Self>) {
        let reduce = cx.reduce_motion();
        let mut rows: Vec<Row> = Vec::new();
        let mut old = std::mem::take(&mut self.rows);
        for item in items {
            match old.iter().position(|r| r.item.id == item.id && r.exit.is_none()) {
                Some(at) => {
                    let mut row = old.remove(at);
                    let target = clamp_progress(item.progress, item.status) / 100.;
                    if (row.bar.target() - target).abs() > 1e-4 {
                        row.bar.animate(target, Curve::Ease(BAR_TIME, ease::OUT), 0., reduce);
                    }
                    if row.item.status != item.status && !reduce {
                        let mut run = Channel::new(0.);
                        run.animate(1., Curve::Ease(SWAP_TIME * 2., ease::OUT), 0., false);
                        row.swap = Some((row.item.status, run));
                    }
                    row.item = item;
                    rows.push(row);
                }
                None => rows.push(Self::new_row(item, reduce)),
            }
        }
        // What left stays where it was, leaving, until its exit has run.
        for mut gone in old {
            if gone.exit.is_none() {
                if reduce {
                    continue;
                }
                let mut run = Channel::new(0.);
                run.animate(1., Curve::Ease(ROW_TIME, ease::OUT), 0., false);
                gone.exit = Some(run);
            }
            let at = rows.len().min(rows.iter().position(|r| r.item.id == gone.item.id).unwrap_or(rows.len()));
            rows.insert(at, gone);
        }
        self.rows = rows;
        cx.notify();
    }

    /// Changes one file in place, for progress and status as the owner sends it.
    pub fn update(&mut self, id: &str, change: impl FnOnce(&mut UploadItem), cx: &mut Context<Self>) {
        let mut items = self.items();
        if let Some(item) = items.iter_mut().find(|i| i.id == id) {
            change(item);
            self.set_items(items, cx);
        }
    }

    /// Puts files on the queue, as uploading, within what the room and the accepted kinds allow.
    pub fn add_paths(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let room = self.max_files.map(|max| max.saturating_sub(self.live()));
        if room == Some(0) {
            return;
        }
        let taken = take_paths(paths, &self.accept, room, self.multiple);
        let added: Vec<UploadItem> = taken.iter().enumerate().filter_map(|(i, p)| UploadItem::from_path(p, i).ok()).map(|mut i| {
            i.status = UploadStatus::Uploading;
            i
        }).collect();
        if added.is_empty() {
            return;
        }
        let mut items = self.items();
        items.extend(added.clone());
        self.set_items(items, cx);
        cx.emit(FileUploadEvent::Added(added));
    }

    pub(super) fn remove(&mut self, id: &SharedString, cx: &mut Context<Self>) {
        let mut items = self.items();
        if let Some(at) = items.iter().position(|i| &i.id == id) {
            let gone = items.remove(at);
            self.set_items(items, cx);
            cx.emit(FileUploadEvent::Removed(gone));
        }
    }

    pub(super) fn retry(&mut self, id: &SharedString, cx: &mut Context<Self>) {
        let mut items = self.items();
        if let Some(item) = items.iter_mut().find(|i| &i.id == id) {
            item.error = None;
            item.progress = 0.;
            item.status = UploadStatus::Uploading;
            let again = item.clone();
            self.set_items(items, cx);
            cx.emit(FileUploadEvent::Retried(again));
        }
    }

    fn browse(&mut self, cx: &mut Context<Self>) {
        if self.disabled || self.maxed() {
            return;
        }
        let picked = cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: self.multiple, prompt: Some("Add".into()) });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = picked.await {
                this.update(cx, |s, cx| s.add_paths(&paths, cx)).ok();
            }
        })
        .detach();
    }
}

impl Render for FileUpload {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let this = cx.entity().downgrade();
        // Rows whose exit has run are gone.
        self.rows.retain(|r| r.exit.as_ref().is_none_or(|run| run.is_running()));
        for row in &mut self.rows {
            if row.swap.as_ref().is_some_and(|(_, run)| !run.is_running()) {
                row.swap = None;
            }
        }
        let uploading = self.rows.iter().any(|r| r.item.status == UploadStatus::Uploading);
        let moving = self.rows.iter().any(|r| r.enter.is_running() || r.exit.is_some() || r.bar.is_running() || r.swap.is_some());
        if moving || uploading {
            window.request_animation_frame();
        }
        let spin = (now().saturating_duration_since(crate::motion::epoch()).as_secs_f32() % duration::SPIN.as_secs_f32()) / duration::SPIN.as_secs_f32();
        let centered = self.variant == UploadVariant::Centered;
        let maxed = self.maxed();
        let off = self.disabled || maxed;
        let live = self.live();
        let (title, description): (SharedString, SharedString) = if maxed {
            ("Upload limit reached".into(), format!("{live} of {} files added", self.max_files.unwrap_or(live)).into())
        } else {
            (self.title.clone(), self.description.clone())
        };
        let border = theme.foreground.opacity(0.2);
        let dropzone = {
            let (pick, drop) = (this.clone(), this.clone());
            let strong = theme.foreground;
            div()
                .id((self.id.clone(), "dropzone"))
                .debug_selector(|| "upload-dropzone".into())
                .flex()
                .w_full()
                .overflow_hidden()
                .rounded(px(24.))
                .border_1()
                .border_dashed()
                .border_color(border)
                .bg(theme.background)
                .when(centered, |d| d.min_h(px(224.)).flex_col().items_center().justify_center().gap(px(12.)).p(px(28.)).text_center())
                .when(!centered, |d| d.items_center().gap(px(16.)).p(px(20.)))
                .when(off, |d| d.opacity(0.55))
                .when(!off, |d| {
                    d.cursor_pointer()
                        .hover(|s| s.border_color(strong.opacity(0.4)))
                        .drag_over::<ExternalPaths>(move |s, _, _, _| s.border_color(strong))
                        .on_drop(move |paths: &ExternalPaths, _, cx| {
                            drop.update(cx, |s, cx| s.add_paths(paths.paths(), cx)).ok();
                        })
                        .on_click(move |_, _, cx| {
                            pick.update(cx, |s, cx| s.browse(cx)).ok();
                        })
                })
                .child(
                    div()
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(theme.card_strong)
                        .text_color(theme.foreground)
                        .when(centered, |d| d.size(px(64.)).rounded(px(21.6)).border_1().border_color(theme.foreground.opacity(0.08)))
                        .when(!centered, |d| d.size(px(56.)).rounded(px(20.)))
                        .child(Icon::new(IconName::CloudUpload).size(px(if centered { 28. } else { 24. }))),
                )
                .child(
                    div()
                        .min_w_0()
                        .when(centered, |d| d.max_w(px(320.)))
                        .when(!centered, |d| d.flex_1())
                        .child(div().font_weight(FontWeight::SEMIBOLD).text_color(theme.foreground).text_size(if centered { TextSize::Base.font_size() } else { TextSize::Sm.font_size() }).line_height(px(if centered { 24. } else { 20. })).child(title))
                        .child(
                            div()
                                .mt(px(if centered { 4. } else { 2. }))
                                .text_size(TextSize::Xs.font_size())
                                .line_height(px(if centered { 20. } else { 16. }))
                                .text_color(theme.muted_foreground)
                                .child(description),
                        ),
                )
                .child(
                    div()
                        .flex_none()
                        .when(centered, |d| d.mt(px(4.)).px(px(16.)).py(px(8.)))
                        .when(!centered, |d| d.px(px(14.)).py(px(8.)))
                        .rounded_full()
                        .border_1()
                        .border_color(theme.foreground.opacity(0.12))
                        .text_size(TextSize::Xs.font_size())
                        .line_height(px(16.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.foreground)
                        .child(self.browse_label.clone()),
                )
        };

        let rows = self.rows.iter().map(|row| {
            let item = &row.item;
            let (enter, exit) = (row.enter.value().clamp(0., 1.), row.exit.as_ref().map(|r| r.value().clamp(0., 1.)));
            let opacity = enter * (1. - exit.unwrap_or(0.));
            let offset = ROW_RISE * (1. - enter) - ROW_LEAVE * exit.unwrap_or(0.);
            let status = item.status;
            let ratio = row.bar.value().clamp(0., 1.);
            let show_bar = matches!(status, UploadStatus::Uploading | UploadStatus::Success);
            let id = item.id.clone();
            let (remove_this, remove_id) = (this.clone(), id.clone());
            let (retry_this, retry_id) = (this.clone(), id.clone());
            let live_row = exit.is_none();
            // The status mark: the old one leaves up in the first half of the swap, the new one arrives from below in the second.
            let mark = match &row.swap {
                Some((old, run)) => {
                    let s = run.value().clamp(0., 1.);
                    if s < 0.5 {
                        let t = s * 2.;
                        div().relative().top(px(-SWAP_SHIFT * t)).opacity(1. - t).child(status_mark(&theme, *old, spin)).into_any_element()
                    } else {
                        let t = (s - 0.5) * 2.;
                        div().relative().top(px(SWAP_SHIFT * (1. - t))).opacity(t).child(status_mark(&theme, status, spin)).into_any_element()
                    }
                }
                None => status_mark(&theme, status, spin).into_any_element(),
            };
            let meta = {
                let mut m = format!("{} · {}", kind_of(item), format_bytes(item.size));
                if status == UploadStatus::Error && let Some(error) = &item.error {
                    m.push_str(" · ");
                    m.push_str(error);
                }
                m
            };
            shifted(
                (self.id.clone(), format!("row-{id}")),
                div()
                    .id((self.id.clone(), format!("item-{id}")))
                    .debug_selector({
                        let id = id.clone();
                        move || format!("upload-row-{id}")
                    })
                    .relative()
                    .top(px(offset))
                    .opacity(opacity)
                    .overflow_hidden()
                    .rounded(px(16.))
                    .border_1()
                    .border_color(theme.foreground.opacity(0.08))
                    .bg(theme.background)
                    .p(px(12.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(12.))
                            .child(
                                div()
                                    .flex_none()
                                    .size(px(44.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(12.))
                                    .bg(theme.card_strong)
                                    .text_color(theme.muted_foreground)
                                    .child(Icon::new(icon_of(item)).size(px(20.))),
                            )
                            .child(
                                div()
                                    .min_w_0()
                                    .flex_1()
                                    .child(
                                        div()
                                            .flex()
                                            .items_start()
                                            .justify_between()
                                            .gap(px(12.))
                                            .child(
                                                div()
                                                    .min_w_0()
                                                    .child(div().truncate().text_size(TextSize::Sm.font_size()).line_height(px(20.)).font_weight(FontWeight::MEDIUM).text_color(theme.foreground).child(item.name.clone()))
                                                    .child(div().mt(px(2.)).text_size(TextSize::Xs.font_size()).line_height(px(16.)).text_color(theme.muted_foreground).child(SharedString::from(meta))),
                                            )
                                            .child(
                                                div()
                                                    .flex_none()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(4.))
                                                    .child(div().size(px(24.)).flex().items_center().justify_center().child(mark))
                                                    .when(status == UploadStatus::Error && live_row, |d| {
                                                        let retry_id = retry_id.clone();
                                                        d.child(round_button(&theme, (self.id.clone(), format!("retry-{id}")).into(), IconName::RotateLeft, "upload-retry").on_click(move |_, _, cx| {
                                                            cx.stop_propagation();
                                                            retry_this.update(cx, |s, cx| s.retry(&retry_id, cx)).ok();
                                                        }))
                                                    })
                                                    .when(live_row, |d| {
                                                        let remove_id = remove_id.clone();
                                                        d.child(round_button(&theme, (self.id.clone(), format!("remove-{id}")).into(), IconName::Close, "upload-remove").on_click(move |_, _, cx| {
                                                            cx.stop_propagation();
                                                            remove_this.update(cx, |s, cx| s.remove(&remove_id, cx)).ok();
                                                        }))
                                                    }),
                                            ),
                                    )
                                    .when(show_bar, |d| {
                                        d.child(
                                            div().mt(px(12.)).child(
                                                CellBar::new(Some(ratio))
                                                    .stretch(true)
                                                    .cells(UPLOAD_CELLS)
                                                    .color(if status == UploadStatus::Success { theme.success } else { theme.foreground })
                                                    .debug_name(format!("upload-bar-{id}")),
                                            ),
                                        )
                                    }),
                            ),
                    ),
            )
            .spring(Spring::LAYOUT)
            .into_any_element()
        });

        div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .gap(px(12.))
            .w_full()
            .child(dropzone)
            .child(div().flex().flex_col().gap(px(8.)).children(rows))
    }
}
