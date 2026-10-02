use std::{rc::Rc, sync::Arc};

use gpui_kit::{
    App, ElementId, InteractiveElement, IntoElement, ListHorizontalSizingBehavior,
    ListSizingBehavior, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, UniformListScrollHandle, Window, div, prelude::FluentBuilder, uniform_list,
};

use crate::scale::px;
use crate::{
    copy_feedback::CopyFeedback,
    file_icon::FileIcon,
    focus::PressStop,
    icon::{Icon, IconName},
    motion::duration,
    reveal::Reveal,
    syntax::Side,
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::types::{DiffLineKind, FileDiffStatus, MAX_HEIGHT, ROW_HEIGHT};
use super::helpers::{diff_row, diff_stats, follow_status, hunk_starts};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    /// The old-file gutter number: set for context and removed lines.
    pub old_line: Option<u32>,
    /// The new-file gutter number: set for context and added lines.
    pub new_line: Option<u32>,
    pub text: SharedString,
}

impl DiffLine {
    /// Parses unified diff text, numbering lines from each hunk header. Everything before the first hunk
    /// (`diff --git`, `index`, `---`, `+++`) is dropped, and so are `\ No newline at end of file` markers.
    /// A later `diff --git` line (a second file in the same patch) drops back out of the hunk, so that
    /// file's own `---`/`+++` headers are dropped too instead of being read as removed/added lines.
    pub fn parse(diff: &str) -> Vec<DiffLine> {
        let (mut old, mut new) = (0u32, 0u32);
        let mut in_hunk = false;
        let mut lines = Vec::new();
        for raw in diff.lines() {
            if raw.starts_with("diff ") {
                in_hunk = false;
                continue;
            }
            let (kind, old_line, new_line) = if let Some(rest) = raw.strip_prefix("@@") {
                in_hunk = true;
                (old, new) = hunk_starts(rest);
                (DiffLineKind::Hunk, None, None)
            } else if !in_hunk || raw.starts_with('\\') {
                continue;
            } else if raw.starts_with('+') {
                new += 1;
                (DiffLineKind::Added, None, Some(new))
            } else if raw.starts_with('-') {
                old += 1;
                (DiffLineKind::Removed, Some(old), None)
            } else {
                old += 1;
                new += 1;
                (DiffLineKind::Context, Some(old), Some(new))
            };
            let text = match kind {
                DiffLineKind::Hunk => raw,
                _ => raw.get(1..).unwrap_or(""),
            };
            lines.push(DiffLine { kind, old_line, new_line, text: text.to_string().into() });
        }
        lines
    }
}

#[derive(IntoElement)]
pub struct FileDiff {
    id: ElementId,
    pub(super) path: SharedString,
    pub(super) lines: Vec<DiffLine>,
    pub(super) status: FileDiffStatus,
    default_open: bool,
    pub(super) collapse_on_complete: bool,
    max_height: f32,
    pub(super) copy_text: Option<SharedString>,
    scroll: Option<UniformListScrollHandle>,
}

impl FileDiff {
    pub fn new(id: impl Into<ElementId>, path: impl Into<SharedString>, lines: Vec<DiffLine>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            lines,
            status: FileDiffStatus::Streaming,
            default_open: true,
            collapse_on_complete: true,
            max_height: MAX_HEIGHT,
            copy_text: None,
            scroll: None,
        }
    }

    /// Scrolls the rows with a handle the owner keeps, to move them from outside.
    pub fn scroll_handle(mut self, handle: UniformListScrollHandle) -> Self {
        self.scroll = Some(handle);
        self
    }

    pub fn status(mut self, status: FileDiffStatus) -> Self {
        self.status = status;
        self
    }

    /// Opens on first render. The user's choice, and `collapse_on_complete`, win after that.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Closes itself the moment `status` goes from `Streaming` to `Complete`. On by default.
    pub fn collapse_on_complete(mut self, collapse: bool) -> Self {
        self.collapse_on_complete = collapse;
        self
    }

    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }

    /// Shows the Copy button and what it copies. Without this the footer row is omitted.
    pub fn copy_text(mut self, text: impl Into<SharedString>) -> Self {
        self.copy_text = Some(text.into());
        self
    }
}

pub(super) struct DiffMotion {
    pub(super) status: FileDiffStatus,
    pub(super) disclosure: Reveal,
    copy: CopyFeedback,
    scroll: UniformListScrollHandle,
}

impl RenderOnce for FileDiff {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let status = self.status;
        let default_open = self.default_open;
        let motion = window.use_keyed_state(self.id.clone(), cx, move |_, _| DiffMotion {
            status,
            disclosure: Reveal::new(default_open),
            copy: CopyFeedback::default(),
            scroll: UniformListScrollHandle::new(),
        });
        follow_status(&motion, status, self.collapse_on_complete, reduce, cx);
        let m = motion.read(cx);
        let streaming = status == FileDiffStatus::Streaming;
        if m.disclosure.is_moving() || (streaming && !reduce) {
            window.request_animation_frame();
        }
        let (reveal, chevron, copied) = (m.disclosure.reveal.value(), m.disclosure.chevron.value(), m.copy.copied());
        let scroll = self.scroll.clone().unwrap_or_else(|| m.scroll.clone());
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let (added, removed) = diff_stats(&self.lines);
        let child = |name: &'static str| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());
        // Each side as one text, so a comment or a string across rows keeps its colour.
        let row_sides = crate::syntax::row_sides(&self.lines);
        let side_runs = crate::syntax::language_for(&self.path).map(|language| {
            let (old, new) = crate::syntax::sides(&self.lines);
            (
                crate::syntax::highlight(language, &old.text, child("old"), cx),
                crate::syntax::highlight(language, &new.text, child("new"), cx),
            )
        });
        // lucide's `animate-spin`.
        let spin_ms = duration::SPIN.as_millis();
        let spin = if streaming && !reduce {
            (std::time::UNIX_EPOCH.elapsed().unwrap_or_default().as_millis() % spin_ms) as f32 / spin_ms as f32
        } else {
            0.
        };

        let toggle = motion.clone();
        let header = div()
            .id(child("header"))
            .group("file-diff-header")
            .flex()
            .items_center()
            .gap(px(8.))
            .min_h(px(36.))
            .w_full()
            .py(px(4.))
            .rounded(radius::md())
            .cursor_pointer()
            .press_stop((self.id.clone(), "head-focus"), crate::theme::radius::md(), window, cx)
            .on_click(move |_, _, cx| {
                let reduce = cx.reduce_motion();
                toggle.update(cx, |m, cx| {
                    let open = !m.disclosure.open;
                    m.disclosure.set_open(open, reduce);
                    cx.notify();
                });
            })
            .child(FileIcon::file(&self.path).size(px(16.)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .font_family(MONO_FONT_FAMILY)
                    .text_size(TextSize::Xs.font_size())
                    .text_color(theme.foreground.opacity(0.8))
                    .child(self.path),
            )
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(8.))
                    .when(added > 0, |d| {
                        d.child(
                            div()
                                .font_family(MONO_FONT_FAMILY)
                                .text_size(TextSize::Xs.font_size())
                                .text_color(theme.diff_color(true))
                                .child(format!("+{added}")),
                        )
                    })
                    .when(removed > 0, |d| {
                        d.child(
                            div()
                                .font_family(MONO_FONT_FAMILY)
                                .text_size(TextSize::Xs.font_size())
                                .text_color(theme.diff_color(false))
                                .child(format!("\u{2212}{removed}")),
                        )
                    }),
            )
            .child(
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(16.))
                    .text_color(muted.opacity(0.6))
                    .child(if streaming {
                        Icon::new(IconName::Progress).size(px(14.)).turn(spin).into_any_element()
                    } else {
                        Icon::new(IconName::Check).size(px(14.)).into_any_element()
                    }),
            )
            .child(
                div()
                    .flex_none()
                    .text_color(muted.opacity(0.45))
                    .group_hover("file-diff-header", |s| s.text_color(muted))
                    .child(Icon::new(IconName::ChevronDown).size(px(14.)).turn(chevron / 360.)),
            );

        let lines: Rc<[DiffLine]> = self.lines.into();
        let row_sides: Rc<[Option<(Side, usize)>]> = row_sides.into();
        // The widest row sets the list's width, so long lines scroll sideways.
        let widest = lines.iter().enumerate().max_by_key(|(_, l)| l.text.len()).map(|(i, _)| i);
        let row_theme = theme.clone();
        let rows = uniform_list(child("rows"), lines.len(), move |range, _, _| {
            range
                .map(|i| {
                    let runs = row_sides[i].zip(side_runs.as_ref()).and_then(|((side, at), (old, new))| {
                        let lines = match side {
                            Side::Old => old.as_ref(),
                            Side::New => new.as_ref(),
                        }?;
                        lines.get(at).cloned()
                    });
                    diff_row(&lines[i], runs, &row_theme)
                })
                .collect()
        })
        .track_scroll(&scroll)
        .with_sizing_behavior(ListSizingBehavior::Infer)
        .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
        .with_width_from_item(widest)
        .max_h(px(self.max_height))
        .font_family(MONO_FONT_FAMILY)
        .text_size(TextSize::Xs.font_size())
        .line_height(px(ROW_HEIGHT));

        let footer = self.copy_text.map(|text| {
            let copy_state = motion.clone();
            div().flex().justify_end().px(px(8.)).pb(px(6.)).pt(px(4.)).child(
                div()
                    .id(child("copy"))
                    .flex()
                    .size(px(28.))
                    .items_center()
                    .justify_center()
                    .rounded(radius::md())
                    .cursor_pointer()
                    .text_color(muted)
                    .hover(|s| s.bg(theme.background.opacity(0.7)).text_color(theme.foreground))
                    .press_stop((self.id.clone(), "copy-focus"), crate::theme::radius::md(), window, cx)
                    .on_click(move |_, _, cx| {
                        CopyFeedback::click(&copy_state, |m: &mut DiffMotion| &mut m.copy, text.to_string(), cx);
                    })
                    .child(Icon::new(if copied { IconName::Check } else { IconName::Copy }).size(px(14.))),
            )
        });

        let card = div()
            .flex()
            .flex_col()
            .overflow_hidden()
            .rounded(radius::xl())
            .bg(theme.card.opacity(0.8))
            .child(rows)
            .when_some(footer, |d, footer| d.child(footer));

        div()
            .flex()
            .flex_col()
            .w_full()
            .text_size(TextSize::Sm.font_size())
            .line_height(TextSize::Sm.line_height())
            .child(header)
            .when_some((reveal > 0.001).then_some(card), |d, card| {
                d.child(div().pl(px(24.)).pt(px(6.)).child(div().relative().top(px(-4. * (1. - reveal))).opacity(reveal).child(card)))
            })
    }
}
