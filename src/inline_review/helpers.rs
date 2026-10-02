use super::{AcceptHunk, RejectHunk};

use std::ops::Range;

use gpui_kit::{
    App,
    ElementId,
    Entity,
    Hsla,
    InteractiveElement,
    IntoElement,
    KeyBinding,
    MouseButton,
    ParentElement,
    SharedString,
    StatefulInteractiveElement,
    Styled,
    Window,
    base::input,
    component::input::EditorState,
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::{Icon, IconName},
    theme::radius,
    tooltip::Tooltip,
};
use super::structs::InlineHunk;
use super::types::{ACCEPT_KEYS, COMPACT_BELOW, DecideHandler, Decision, REJECT_KEYS, RowHandler};

/// The keys the bar's hints name. They act on the hunk under the caret; with the caret outside every
/// hunk they keep their usual editing meaning.
pub(crate) fn bind_keys(cx: &mut App) {
    let context = Some("InlineReview > Input");
    cx.bind_keys([
        KeyBinding::new("secondary-enter", AcceptHunk, context),
        KeyBinding::new("secondary-backspace", RejectHunk, context),
    ]);
}

/// The row ranges to delete, ordered last first.
///
/// `replace_text_in_ranges` is `pub(crate)` in gpui-base, so a multi-hunk decision is a loop of
/// single-range edits. Applying them from the bottom of the file upwards means no edit shifts a range
/// that has not run yet. Empty ranges are dropped, since a pure insertion has nothing to delete.
pub fn plan_edits(decisions: &[(InlineHunk, Decision)]) -> Vec<Range<usize>> {
    let mut ranges: Vec<Range<usize>> = decisions
        .iter()
        .map(|(hunk, decision)| hunk.closing(*decision))
        .filter(|range| !range.is_empty())
        .collect();
    ranges.sort_by_key(|range| std::cmp::Reverse(range.start));
    ranges
}

/// What `decisions` leave of `text`: the same edits [`apply`] runs on the buffer, on a string.
pub fn apply_to_text(text: &str, decisions: &[(InlineHunk, Decision)]) -> String {
    let mut out = text.to_string();
    for rows in plan_edits(decisions) {
        out.replace_range(rows_to_bytes(&out, &rows), "");
    }
    out
}

/// Where the caret lands after those rows go.
///
/// The caret must never jump to the top of the file, which is what a whole-text rewrite would do. A
/// caret below a deleted block moves up by that block's height. A caret inside one lands where the
/// block started, which is the nearest row that still exists.
pub fn hold_caret(caret_row: usize, closings: &[Range<usize>]) -> usize {
    let mut row = caret_row;
    for range in closings.iter().filter(|r| !r.is_empty()) {
        if caret_row >= range.end {
            row -= range.len();
        } else if range.contains(&caret_row) {
            row -= caret_row - range.start;
        }
    }
    row
}

/// The byte offset a row starts at, for a range the editor will accept.
///
/// gpui-base exposes no public row-to-offset helper, so this counts it from the text. The offset of
/// the row one past the last is the length of the text, so a hunk that ends the file still has a
/// range to delete.
pub fn row_to_byte(text: &str, row: usize) -> usize {
    if row == 0 {
        return 0;
    }
    let mut seen = 0;
    for (offset, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            seen += 1;
            if seen == row {
                return offset + 1;
            }
        }
    }
    text.len()
}

/// One row range as a byte range, ready for `set_selected_range`.
pub fn rows_to_bytes(text: &str, rows: &Range<usize>) -> Range<usize> {
    row_to_byte(text, rows.start)..row_to_byte(text, rows.end)
}

/// The row ranges to wash, each marked with whether it is the agent's side. Empty sides are
/// dropped: a pure insertion has no old rows to show. The editor clips them to the screen.
pub fn washes(hunks: &[InlineHunk]) -> Vec<(Range<usize>, bool)> {
    hunks
        .iter()
        .flat_map(|hunk| [(hunk.removed.clone(), false), (hunk.added.clone(), true)])
        .filter(|(rows, _)| !rows.is_empty())
        .collect()
}

/// The remaining hunks after one hunk's rows went from the buffer.
///
/// Deleting rows moves every row below them up, so a hunk below the deletion must slide up by the
/// same height or its bands and its bar land on the wrong code. A hunk above it does not move, and
/// the decided hunk itself is dropped.
pub fn shift_after(hunks: &[InlineHunk], decided: &SharedString, closed: &Range<usize>) -> Vec<InlineHunk> {
    let height = closed.len();
    hunks
        .iter()
        .filter(|hunk| &hunk.id != decided)
        .map(|hunk| {
            if height == 0 || hunk.removed.start < closed.end {
                return hunk.clone();
            }
            InlineHunk {
                id: hunk.id.clone(),
                removed: (hunk.removed.start - height)..(hunk.removed.end - height),
                added: (hunk.added.start - height)..(hunk.added.end - height),
            }
        })
        .collect()
}

/// The hunks after the user edited the buffer from `before` to `after`, so every band and bar stays
/// on the rows it described.
///
/// The edit is found as the rows between the lines both texts share at the top and at the bottom.
/// Rows above it keep their place, rows below it move by the change in height, and a hunk the edit
/// lands in grows or shrinks with it. Two rules settle a line added exactly on a boundary: one added
/// at a hunk's first row pushes the hunk down, and one added between the old and the new rows joins
/// the agent's side. A hunk whose rows were all deleted is dropped.
pub fn track_edit(hunks: &[InlineHunk], before: &str, after: &str) -> Vec<InlineHunk> {
    let old: Vec<&str> = before.split('\n').collect();
    let new: Vec<&str> = after.split('\n').collect();
    let top = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let room = old.len().min(new.len()) - top;
    let bottom = old.iter().rev().zip(new.iter().rev()).take(room).take_while(|(a, b)| a == b).count();
    let (old_end, new_end) = (old.len() - bottom, new.len() - bottom);
    let inserted_here = top == old_end;
    // `pushed` is for a boundary that a line added right there moves down.
    let map = |row: usize, pushed: bool| -> usize {
        if row < top || (row == top && !(pushed && inserted_here)) {
            row
        } else if row >= old_end {
            row + new_end - old_end
        } else {
            row.min(new_end)
        }
    };
    hunks
        .iter()
        .filter_map(|hunk| {
            let start = map(hunk.removed.start, true);
            let middle = map(hunk.removed.end, false).max(start);
            let end = map(hunk.added.end, false).max(middle);
            (start < end).then(|| InlineHunk { id: hunk.id.clone(), removed: start..middle, added: middle..end })
        })
        .collect()
}

/// Whether the hunk bar is the short one at this width of the review.
pub fn compact_bar(width: f32) -> bool {
    width < COMPACT_BELOW
}

/// The hunk whose rows hold `row`, which is the one the keyboard decides.
pub fn hunk_at_row(hunks: &[InlineHunk], row: usize) -> Option<&InlineHunk> {
    hunks.iter().find(|hunk| hunk.rows().contains(&row))
}

/// How many hunks still wait on the user.
pub fn pending_count(hunks: &[InlineHunk], decided: &[SharedString]) -> usize {
    hunks.iter().filter(|hunk| !decided.contains(&hunk.id)).count()
}

// ---------------------------------------------------------------------------
// The element. Everything above is pure; everything below draws it.
// ---------------------------------------------------------------------------

/// The row a byte offset sits on, so the caret can be put back where it was.
pub fn byte_to_row(text: &str, offset: usize) -> usize {
    text.as_bytes().iter().take(offset).filter(|b| **b == b'\n').count()
}

/// Applies the user's decisions to the buffer.
///
/// The byte ranges are all measured against the text as it is now, which is only safe because
/// [`plan_edits`] orders them from the bottom of the file upwards: an edit low in the file cannot move
/// an offset above it. Each `replace` is one undo step, so the user can take a decision back.
///
/// The caret keeps its row rather than jumping to the top, which is what a whole-text rewrite does.
pub fn apply(
    state: &Entity<EditorState>,
    decisions: &[(InlineHunk, Decision)],
    window: &mut Window,
    cx: &mut App,
) {
    let rows = plan_edits(decisions);
    if rows.is_empty() {
        return;
    }
    state.update(cx, |state, cx| {
        let text = state.text().to_string();
        let caret_row = byte_to_row(&text, state.cursor());
        let ranges: Vec<Range<usize>> = rows.iter().map(|range| rows_to_bytes(&text, range)).collect();
        for range in ranges {
            state.set_selected_range(range, cx);
            state.replace("", window, cx);
        }
        let landed = hold_caret(caret_row, &rows);
        let offset = row_to_byte(&state.text().to_string(), landed);
        state.set_selected_range(offset..offset, cx);
    });
}

/// Decides the hunk under the caret, or does what the key does in any editor when there is none.
pub(super) fn decide_at_caret(
    state: &Entity<EditorState>,
    hunks: &[InlineHunk],
    on_decide: &Option<DecideHandler>,
    decision: Decision,
    window: &mut Window,
    cx: &mut App,
) {
    let read = state.read(cx);
    let row = byte_to_row(&read.text().to_string(), read.cursor());
    match (hunk_at_row(hunks, row), on_decide) {
        (Some(hunk), Some(decide)) => decide(&hunk.id, decision, window, cx),
        _ => match decision {
            Decision::Accept => window.dispatch_action(Box::new(input::Enter { secondary: true, shift: false }), cx),
            Decision::Reject if cfg!(target_os = "macos") => {
                window.dispatch_action(Box::new(input::DeleteToBeginningOfLine), cx)
            }
            Decision::Reject => window.dispatch_action(Box::new(input::DeleteToPreviousWordStart), cx),
        },
    }
}

/// The gutter's "+" on the row under the pointer. It takes the press, so the caret stays put; it does not
/// block the pointer, so the editor still knows which row the pointer is over.
pub(super) fn add_comment_button(row: usize, fill: Hsla, ink: Hsla, add: RowHandler) -> impl IntoElement {
    div()
        .id(("add-comment", row))
        .flex()
        .size(px(16.))
        .items_center()
        .justify_center()
        .rounded(radius::md() - px(2.))
        .bg(fill)
        .text_color(ink)
        .cursor_pointer()
        .tooltip(Tooltip::text("Add a comment"))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            add(row, window, cx);
        })
        .child(Icon::new(IconName::Add).size(px(12.)))
}

/// Accept and Reject for one hunk. The editor places it at the right end of the hunk's first row.
pub(super) fn hunk_bar(hunk: InlineHunk, is_current: bool, compact: bool, bar_fill: Hsla, on_decide: Option<DecideHandler>) -> impl IntoElement {
    let accept_id = hunk.id.clone();
    let reject_id = hunk.id.clone();
    let accept = on_decide.clone();
    let reject = on_decide;

    div()
        .flex()
        .items_center()
        .gap_1()
        .p_0p5()
        .rounded(radius::md())
        .bg(bar_fill)
        // A control, not text: the arrow over the bar, and each button's own hand over it.
        .cursor_default()
        // Borderless: the bar reads as a raised fill, not a framed box.
        .debug_selector(|| "hunk-bar".into())
        .when(!is_current, |d| d.invisible().group_hover("inline-hunk", |d| d.visible()))
        .child(
            Button::new(ElementId::Name(format!("accept-{}", hunk.id).into()))
                .when(!compact, |b| b.label("Accept").cap(ACCEPT_KEYS))
                .when(compact, |b| b.icon(IconName::Check).tooltip(format!("Accept  {ACCEPT_KEYS}")))
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Primary)
                .on_click(move |_, window, cx| {
                    if let Some(f) = &accept {
                        f(&accept_id, Decision::Accept, window, cx);
                    }
                }),
        )
        .child(
            Button::new(ElementId::Name(format!("reject-{}", hunk.id).into()))
                .when(!compact, |b| b.label("Reject").cap(REJECT_KEYS))
                .when(compact, |b| b.icon(IconName::Close).tooltip(format!("Reject  {REJECT_KEYS}")))
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Secondary)
                .on_click(move |_, window, cx| {
                    if let Some(f) = &reject {
                        f(&reject_id, Decision::Reject, window, cx);
                    }
                }),
        )
}
