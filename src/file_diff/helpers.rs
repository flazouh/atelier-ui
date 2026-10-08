use gpui_kit::{
    AnyElement, App, Entity, IntoElement, ParentElement, Styled, StyledText, div,
    prelude::FluentBuilder,
};

use super::structs::{DiffLine, DiffMotion};
use super::types::{DiffLineKind, FileDiffStatus, ROW_HEIGHT};
use crate::scale::px;
use crate::{
    syntax::LineRuns,
    theme::{Theme, radius},
};

/// The line before each side's first line, from ` -12,4 +12,5 @@`, so the first line gets number 12.
pub(super) fn hunk_starts(header: &str) -> (u32, u32) {
    let start = |sign: char| {
        header
            .split_whitespace()
            .find_map(|part| part.strip_prefix(sign))
            .and_then(|range| range.split(',').next()?.parse::<u32>().ok())
            .map_or(0, |n| n.saturating_sub(1))
    };
    (start('-'), start('+'))
}

/// The diff's fill: a box on the panel, or a wash of the ink over the box it sits in, such as an approval.
pub(super) fn fill(theme: &Theme, inset: bool) -> gpui_kit::Hsla {
    if inset {
        theme.wash()
    } else {
        theme.card_strong
    }
}

/// How many lines were added and removed.
pub fn diff_stats(lines: &[DiffLine]) -> (usize, usize) {
    let count = |kind| lines.iter().filter(|l| l.kind == kind).count();
    (count(DiffLineKind::Added), count(DiffLineKind::Removed))
}

/// Opens when streaming starts and closes when it completes, like beui's `collapseOnComplete` effect.
pub(super) fn follow_status(
    motion: &Entity<DiffMotion>,
    status: FileDiffStatus,
    collapse_on_complete: bool,
    reduce: bool,
    cx: &mut App,
) {
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

/// One row: the two line numbers, the sign, and the text with its colours, [`ROW_HEIGHT`] tall.
/// `faint` is `theme.faint()`, which searches for a colour: the caller works it out once for all the rows.
pub(super) fn diff_row(
    line: &DiffLine,
    runs: Option<LineRuns>,
    theme: &Theme,
    faint: gpui_kit::Hsla,
    round_bottom: bool,
) -> AnyElement {
    let muted = theme.muted_foreground;
    let num_col = |n: Option<u32>| {
        div()
            .w(px(36.))
            .flex_none()
            .pr(px(8.))
            .flex()
            .justify_end()
            .text_color(faint)
            .children(n.map(|n| n.to_string()))
    };
    let (bg, sign, sign_color) = match line.kind {
        DiffLineKind::Added => (Some(theme.diff_line(true)), "+", theme.diff_color(true)),
        DiffLineKind::Removed => (
            Some(theme.diff_line(false)),
            "\u{2212}",
            theme.diff_color(false),
        ),
        DiffLineKind::Context | DiffLineKind::Hunk => (None, "", faint),
    };
    let hunk = line.kind == DiffLineKind::Hunk;
    let text = line.text.clone();
    div()
        .flex()
        .w_full()
        .h(px(ROW_HEIGHT))
        .when_some(bg, |d, bg| d.bg(bg))
        // The card clips to its rectangle, not to its rounded corners, so the row at the card's bottom rounds its own.
        .when(round_bottom, |d| {
            d.rounded_bl(radius::card()).rounded_br(radius::card())
        })
        .child(num_col(line.old_line))
        .child(num_col(line.new_line))
        .child(
            div()
                .w(px(16.))
                .flex_none()
                .flex()
                .justify_center()
                .text_color(sign_color)
                .child(sign),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .px(px(6.))
                .whitespace_nowrap()
                .text_color(if hunk { muted } else { theme.foreground })
                .child(match runs {
                    Some(runs) => StyledText::new(text)
                        .with_highlights(runs)
                        .into_any_element(),
                    None => text.into_any_element(),
                }),
        )
        .into_any_element()
}

/// The row that stands at the card's bottom edge, when one does: the last, unless something stands under the rows (the copy
/// footer, or the line that says how many rows are hidden) or the rows scroll, which cuts the bottom one at any place.
pub(super) fn bottom_row(
    total: usize,
    hidden_rows_hint: bool,
    footer: bool,
    scrolls: bool,
) -> Option<usize> {
    if footer || hidden_rows_hint || scrolls {
        return None;
    }
    total.checked_sub(1)
}
