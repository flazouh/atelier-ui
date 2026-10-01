//! beui's FileDiff (`components/agents/file-diff.tsx`), class for class:
//!
//! - Root `w-full text-sm`. Header `min-h-9 gap-2 rounded-md py-1`: a `size-4` file icon, the path
//!   `text-xs` at 80% foreground, `+n`/`\u{2212}n` change counts, a `size-4` status slot (spinning loader
//!   while streaming, a check once complete), and a rotating `size-3.5` chevron.
//! - Body `pl-6 pt-1.5`: a `rounded-xl` card holding a scrollable two-column line-number gutter (old,
//!   new), a `size-4` sign column, and the line text, then a footer row with Copy when `copy_text` is set.
//! - Rows are a virtual list (`uniform_list`): each is [`ROW_HEIGHT`] tall, and only the rows in view
//!   are built and laid out, so a 5k-row diff costs a frame what a short one does.
//! - It opens while streaming and closes by itself on completion, like beui's `collapseOnComplete`.
//! - Syntax colours as in the editor ([`crate::syntax`]): each side of the diff is highlighted as one whole
//!   text, off the UI thread, cached, and each row reads its side's line. The +/− washes sit under the colours.
use std::{rc::Rc, sync::Arc};

use gpui_kit::{
    AnyElement, App, ElementId, Entity, InteractiveElement, IntoElement, ListHorizontalSizingBehavior, ListSizingBehavior,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, StyledText, UniformListScrollHandle,
    Window, div, prelude::FluentBuilder, uniform_list,
};
use crate::scale::px;

use crate::{
    focus::PressStop,
    file_icon::FileIcon,
    copy_feedback::CopyFeedback,
    icon::{Icon, IconName},
    motion::duration,
    reveal::Reveal,
    syntax::{LineRuns, Side},
    theme::{ActiveTheme, Theme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

/// beui's `maxHeight` for the diff viewport.
const MAX_HEIGHT: f32 = 220.;

/// Every row's height: the virtual list counts on it.
pub const ROW_HEIGHT: f32 = 20.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
    /// A `@@ -1,4 +1,5 @@` hunk header. beui's own `FileDiffLine` has no such row (its callers hand it
    /// pre-split lines); this crate parses raw unified diff text instead, so hunk headers need a row.
    Hunk,
}

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

/// The line before each side's first line, from ` -12,4 +12,5 @@`, so the first line gets number 12.
fn hunk_starts(header: &str) -> (u32, u32) {
    let start = |sign: char| {
        header
            .split_whitespace()
            .find_map(|part| part.strip_prefix(sign))
            .and_then(|range| range.split(',').next()?.parse::<u32>().ok())
            .map_or(0, |n| n.saturating_sub(1))
    };
    (start('-'), start('+'))
}

/// How many lines were added and removed.
pub fn diff_stats(lines: &[DiffLine]) -> (usize, usize) {
    let count = |kind| lines.iter().filter(|l| l.kind == kind).count();
    (count(DiffLineKind::Added), count(DiffLineKind::Removed))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileDiffStatus {
    Streaming,
    Complete,
}

#[derive(IntoElement)]
pub struct FileDiff {
    id: ElementId,
    path: SharedString,
    lines: Vec<DiffLine>,
    status: FileDiffStatus,
    default_open: bool,
    collapse_on_complete: bool,
    max_height: f32,
    copy_text: Option<SharedString>,
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

struct DiffMotion {
    status: FileDiffStatus,
    disclosure: Reveal,
    copy: CopyFeedback,
    scroll: UniformListScrollHandle,
}

/// Opens when streaming starts and closes when it completes, like beui's `collapseOnComplete` effect.
fn follow_status(motion: &Entity<DiffMotion>, status: FileDiffStatus, collapse_on_complete: bool, reduce: bool, cx: &mut App) {
    motion.update(cx, |m, _| {
        if m.status != status {
            let was_streaming = m.status == FileDiffStatus::Streaming;
            m.status = status;
            if status == FileDiffStatus::Streaming {
                m.disclosure.set_open(true, reduce);
            } else if was_streaming && collapse_on_complete {
                m.disclosure.set_open(false, reduce);
            }
        }
    });
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

/// One row: the two line numbers, the sign, and the text with its colours, [`ROW_HEIGHT`] tall.
fn diff_row(line: &DiffLine, runs: Option<LineRuns>, theme: &Theme) -> AnyElement {
    let muted = theme.muted_foreground;
    let num_col = |n: Option<u32>| {
        div().w(px(36.)).flex_none().pr(px(8.)).flex().justify_end().text_color(muted.opacity(0.4)).children(n.map(|n| n.to_string()))
    };
    let (bg, sign, sign_color) = match line.kind {
        DiffLineKind::Added => (Some(theme.diff_line(true)), "+", theme.diff_color(true)),
        DiffLineKind::Removed => (Some(theme.diff_line(false)), "\u{2212}", theme.diff_color(false)),
        DiffLineKind::Context | DiffLineKind::Hunk => (None, "", muted.opacity(0.45)),
    };
    let hunk = line.kind == DiffLineKind::Hunk;
    let text = line.text.clone();
    div()
        .flex()
        .w_full()
        .h(px(ROW_HEIGHT))
        .when_some(bg, |d, bg| d.bg(bg))
        .child(num_col(line.old_line))
        .child(num_col(line.new_line))
        .child(div().w(px(16.)).flex_none().flex().justify_center().text_color(sign_color).child(sign))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .px(px(6.))
                .whitespace_nowrap()
                .text_color(if hunk { muted.opacity(0.7) } else { theme.foreground })
                .child(match runs {
                    Some(runs) => StyledText::new(text).with_highlights(runs).into_any_element(),
                    None => text.into_any_element(),
                }),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests;
