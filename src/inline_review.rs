//! The agent's edits shown inside the file the user is editing.
//!
//! Both sides live in the buffer as real text, the way a conflict marker does: the removed rows, then
//! the added rows. Accepting a hunk deletes the removed rows and leaves the agent's; rejecting it does
//! the mirror. The user can type anywhere at any time, and every decision is one undo step, because
//! the edit goes through `EditorState::replace`.
//!
//! See `docs/inline-review.md` for the gpui-base APIs this is built from, and for the two constraints
//! it works around: a row cannot shrink, so a closing hunk leaves a gap that shrinks instead, and a
//! decoration's background is a glyph-run band rather than a row band.
//!
//! Everything in this half of the file is pure. It takes rows and text and returns rows, ranges and
//! offsets, so the whole decision path is tested without a window.

use std::{collections::HashMap, ops::Range, rc::Rc, sync::Arc, time::Instant};

use gpui_kit::{
    App, ElementId, Entity, Hsla, InteractiveElement, IntoElement, KeyBinding, MouseButton, ParentElement, Pixels,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
    base::input::{self, RowBackground, RowBlock, RowGap, RowWidget},
    component::input::EditorState,
    div,
    prelude::FluentBuilder,
    
};
use crate::scale::px;

use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    code_editor::CodeEditor,
    icon::{Icon, IconName},
    theme::{ActiveTheme, radius},
    tooltip::Tooltip,
};

gpui_kit::actions!(
    inline_review,
    [
        /// Accepts the hunk under the caret.
        AcceptHunk,
        /// Rejects the hunk under the caret.
        RejectHunk,
    ]
);

/// The keys the bar's hints name. They act on the hunk under the caret; with the caret outside every
/// hunk they keep their usual editing meaning.
pub(crate) fn bind_keys(cx: &mut App) {
    let context = Some("InlineReview > Input");
    cx.bind_keys([
        KeyBinding::new("secondary-enter", AcceptHunk, context),
        KeyBinding::new("secondary-backspace", RejectHunk, context),
    ]);
}

/// What pressing a row's "add a comment" button reports: the row.
type RowHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// What a decision on one hunk is reported with.
type DecideHandler = Arc<dyn Fn(&SharedString, Decision, &mut Window, &mut App)>;

/// What the user chose for one hunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Keep the agent's rows and drop the old ones.
    Accept,
    /// Keep the old rows and drop the agent's.
    Reject,
}

/// One hunk as it sits in the buffer.
///
/// `removed` and `added` are buffer row indices, and `removed.end == added.start`: the old rows
/// immediately precede the new ones, so the pair reads as one block. Either side may be empty, which
/// is how a pure insertion and a pure deletion are written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlineHunk {
    pub id: SharedString,
    pub removed: Range<usize>,
    pub added: Range<usize>,
}

impl InlineHunk {
    pub fn new(id: impl Into<SharedString>, removed: Range<usize>, added: Range<usize>) -> Self {
        Self { id: id.into(), removed, added }
    }

    /// Every row the hunk occupies, old and new together.
    pub fn rows(&self) -> Range<usize> {
        self.removed.start..self.added.end
    }

    /// The rows that stay when the user decides.
    pub fn surviving(&self, decision: Decision) -> Range<usize> {
        match decision {
            Decision::Accept => self.added.clone(),
            Decision::Reject => self.removed.clone(),
        }
    }

    /// The rows that go when the user decides.
    pub fn closing(&self, decision: Decision) -> Range<usize> {
        match decision {
            Decision::Accept => self.removed.clone(),
            Decision::Reject => self.added.clone(),
        }
    }
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

/// The review as it stood before and after each decision, so an undo or a redo that brings a
/// decision's text back brings its hunks back too. Undo restores text only; without this a hunk the
/// user took back would come back as plain code with no bands and no bar.
#[derive(Default)]
pub struct DecisionHistory {
    /// `(before, after)` for each decision, oldest first; each side is the text and its hunks.
    decisions: Vec<[(String, Vec<InlineHunk>); 2]>,
}

impl DecisionHistory {
    /// How many decisions are kept. An older one is forgotten, and undoing that far leaves its text
    /// as plain code.
    const KEPT: usize = 64;

    /// Records one decision: the text and hunks before it, and after it.
    pub fn record(&mut self, before: (String, Vec<InlineHunk>), after: (String, Vec<InlineHunk>)) {
        if self.decisions.len() == Self::KEPT {
            self.decisions.remove(0);
        }
        self.decisions.push([before, after]);
    }

    /// The hunks that belong to `text`, when it is the text from just before or just after a
    /// decision, the newest first. `None` for any other text.
    pub fn hunks_for(&self, text: &str) -> Option<Vec<InlineHunk>> {
        self.decisions.iter().rev().flatten().find(|(known, _)| known == text).map(|(_, hunks)| hunks.clone())
    }
}

/// The keys the bar names, as each platform writes them.
const ACCEPT_KEYS: &str = if cfg!(target_os = "macos") { "⌘↵" } else { "⌃↵" };
const REJECT_KEYS: &str = if cfg!(target_os = "macos") { "⌘⌫" } else { "⌃⌫" };

/// Below this width the hunk bar keeps its two icons and drops the words and the key caps, so it does not
/// cover the code it decides. The words and keys move to the tooltips.
pub const COMPACT_BELOW: f32 = 640.;

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

mod resolve;
pub use resolve::Resolve;

#[cfg(test)]
mod tests;

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

/// The agent's edits, laid over the file the user is editing.
///
/// The editor stays writable. A review does not own the buffer, so the user can fix their own typo in
/// the middle of a hunk and then accept it.
#[derive(IntoElement)]
pub struct InlineReview {
    id: ElementId,
    state: Entity<EditorState>,
    hunks: Vec<InlineHunk>,
    current: Option<SharedString>,
    height: Option<Pixels>,
    on_decide: Option<DecideHandler>,
    resolving: Vec<Resolve>,
    on_resolved: Option<DecideHandler>,
    row_blocks: Vec<RowBlock>,
    on_add_comment: Option<RowHandler>,
    on_card: bool,
    decisions: bool,
    read_only: bool,
    fill: bool,
}

impl InlineReview {
    pub fn new(id: impl Into<ElementId>, state: &Entity<EditorState>, hunks: Vec<InlineHunk>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            hunks,
            current: None,
            height: None,
            on_decide: None,
            resolving: Vec::new(),
            on_resolved: None,
            row_blocks: Vec::new(),
            on_add_comment: None,
            on_card: false,
            read_only: false,
            decisions: true,
            fill: false,
        }
    }

    /// Whether each hunk carries Accept and Reject. Off for a diff that is read rather than decided, such
    /// as a pull request's: the washes stay, the bars and the hunk keys go.
    pub fn decisions(mut self, decisions: bool) -> Self {
        self.decisions = decisions;
        self
    }

    /// The editor sits on a card; its closing hunks fade into the card's fill.
    /// Shows the text to read, not to type in, as a pull request's diff: the review's letters then work
    /// from inside it (see [`crate::keys::typing`]).
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn on_card(mut self, on_card: bool) -> Self {
        self.on_card = on_card;
        self
    }

    /// Elements that sit under their rows and push the rows below down, such as a comment thread or a
    /// comment being written.
    pub fn row_blocks(mut self, blocks: Vec<RowBlock>) -> Self {
        self.row_blocks = blocks;
        self
    }

    /// Shows a "+" in the gutter of the row under the pointer; pressing it reports that row.
    pub fn on_add_comment(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_add_comment = Some(Rc::new(f));
        self
    }

    /// Hunks the user decided and their resolves. A fading one loses its bar and its rows fade; once
    /// its fade is over it is handed to [`InlineReview::on_resolved`]. The owner then runs the edit and
    /// calls [`Resolve::edited`], and keeps it here until [`Resolve::is_over`], so its gap can close.
    pub fn resolving(mut self, resolving: Vec<Resolve>) -> Self {
        self.resolving = resolving;
        self
    }

    /// Called once a decided hunk has faded, which is when its text edit should run. The handler marks
    /// the hunk's [`Resolve`] with [`Resolve::edited`]; until it does, this may be called again.
    pub fn on_resolved(mut self, f: impl Fn(&SharedString, Decision, &mut Window, &mut App) + 'static) -> Self {
        self.on_resolved = Some(Arc::new(f));
        self
    }

    /// The hunk the keyboard acts on. Its bar stays up without a hover.
    pub fn current(mut self, id: impl Into<SharedString>) -> Self {
        self.current = Some(id.into());
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    /// Takes the height of its parent instead of a fixed one, as [`CodeEditor::fill`] does.
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    pub fn on_decide(
        mut self,
        f: impl Fn(&SharedString, Decision, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_decide = Some(Arc::new(f));
        self
    }
}

/// Decides the hunk under the caret, or does what the key does in any editor when there is none.
fn decide_at_caret(
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

impl RenderOnce for InlineReview {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        // A decided hunk fades before its edit runs; one whose fade is over goes to `on_resolved`, and
        // stays invisible until its owner marks it edited, so it never flashes back. After the edit its
        // gap closes, and the rows below slide up into the place of the deleted ones.
        let (now, reduce_motion) = (Instant::now(), cx.reduce_motion());
        let mut fades: HashMap<SharedString, (f32, Decision)> = HashMap::new();
        let mut gaps = Vec::new();
        for resolve in &self.resolving {
            if resolve.is_edited() {
                if let Some((row, rows)) = resolve.gap(now, reduce_motion) {
                    gaps.push(RowGap { row, rows });
                    window.request_animation_frame();
                }
                continue;
            }
            let fade = resolve.fade(now, reduce_motion);
            fades.insert(resolve.id.clone(), (fade.unwrap_or(0.), resolve.decision));
            if fade.is_some() {
                window.request_animation_frame();
            } else if let Some(resolved) = self.on_resolved.clone() {
                let (id, decision) = (resolve.id.clone(), resolve.decision);
                window.defer(cx, move |window, cx| resolved(&id, decision, window, cx));
            }
        }
        let deciding = |hunk: &InlineHunk| fades.contains_key(&hunk.id);
        // A cover in the editor's own fill fades the closing rows out, text and all.
        let fill = crate::code_editor::surface(&theme, self.on_card).0;
        let covers = self
            .hunks
            .iter()
            .filter_map(|hunk| {
                let (fade, decision) = fades.get(&hunk.id).copied()?;
                Some(RowBackground { rows: hunk.closing(decision), color: fill.opacity(1. - fade), marker: None })
            })
            .collect();
        // The editor paints the washes and places the bars from its own layout, so they sit exactly
        // on their rows whatever its padding, row height, wrapping or scroll.
        let backgrounds = self
            .hunks
            .iter()
            .flat_map(|hunk| {
                let fade = fades.get(&hunk.id).map_or(1., |(fade, _)| *fade);
                washes(std::slice::from_ref(hunk)).into_iter().map(move |(rows, added)| (rows, added, fade))
            })
            .map(|(rows, added, fade)| RowBackground {
                rows,
                color: theme.diff_line(added).opacity(fade),
                marker: Some(if added { theme.success } else { theme.danger }.opacity(fade)),
            })
            .collect();
        let bar_fill = theme.card_strong;
        // The review's own width in the last frame; the first frame draws the full bar.
        let width = window.use_keyed_state(ElementId::NamedChild(Arc::new(self.id.clone()), "width".into()), cx, |_, _| f32::MAX);
        let compact = compact_bar(*width.read(cx));
        let measure = {
            let width = width.clone();
            gpui_kit::canvas(
                move |bounds, _, cx| {
                    let now = f32::from(bounds.size.width);
                    if (*width.read(cx) - now).abs() > 0.5 {
                        width.update(cx, |w, cx| {
                            *w = now;
                            cx.notify();
                        });
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full()
        };
        let widgets = self
            .hunks
            .iter()
            .filter(|hunk| self.decisions && !deciding(hunk))
            .map(|hunk| {
                let is_current = self.current.as_ref() == Some(&hunk.id);
                let (hunk, on_decide) = (hunk.clone(), self.on_decide.clone());
                RowWidget {
                    row: hunk.rows().start,
                    render: Rc::new(move |_, _| {
                        hunk_bar(hunk.clone(), is_current, compact, bar_fill, on_decide.clone()).into_any_element()
                    }),
                }
            })
            .collect();
        let add = self.on_add_comment.clone().map(|add| -> input::GutterWidget {
            let (fill, ink) = (theme.primary, theme.primary_foreground);
            Rc::new(move |row, _, _| add_comment_button(row, fill, ink, add.clone()).into_any_element())
        });
        self.state.update(cx, |state, cx| {
            state.set_row_backgrounds(backgrounds, cx);
            state.set_row_covers(covers, cx);
            state.set_row_gaps(gaps, cx);
            state.set_row_widgets(widgets);
            state.set_row_blocks(self.row_blocks);
            state.set_gutter_widget(add);
        });
        // A hunk on its way out is no longer the keyboard's to decide.
        let open: Vec<InlineHunk> = self.hunks.iter().filter(|h| self.decisions && !deciding(h)).cloned().collect();
        let keys = (self.state.clone(), open, self.on_decide.clone());
        let accept_keys = keys.clone();

        div()
            .id(self.id)
            .key_context("InlineReview")
            .on_action(move |_: &AcceptHunk, window, cx| {
                let (state, hunks, decide) = &accept_keys;
                decide_at_caret(state, hunks, decide, Decision::Accept, window, cx)
            })
            .on_action(move |_: &RejectHunk, window, cx| {
                let (state, hunks, decide) = &keys;
                decide_at_caret(state, hunks, decide, Decision::Reject, window, cx)
            })
            // The per-hunk bars hide until the pointer is over the review, so the code reads clean.
            .group("inline-hunk")
            .w_full()
            .when_some(self.height, |d, h| d.h(h))
            .when(self.fill, |d| d.h_full())
            .relative()
            .child(measure)
            .child(
                CodeEditor::new(&self.state)
                    .on_card(self.on_card)
                    .read_only(self.read_only)
                    .fill(self.fill)
                    .when_some(self.height, |e, h| e.height(h)),
            )
    }
}

/// The gutter's "+" on the row under the pointer. It takes the press, so the caret stays put; it does not
/// block the pointer, so the editor still knows which row the pointer is over.
fn add_comment_button(row: usize, fill: Hsla, ink: Hsla, add: RowHandler) -> impl IntoElement {
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
fn hunk_bar(hunk: InlineHunk, is_current: bool, compact: bool, bar_fill: Hsla, on_decide: Option<DecideHandler>) -> impl IntoElement {
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
