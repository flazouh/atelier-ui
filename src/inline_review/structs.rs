use super::{AcceptHunk, RejectHunk};

use std::{collections::HashMap, ops::Range, rc::Rc, sync::Arc, time::Instant};

use gpui_kit::{
    App, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window,
    base::input::{self, RowBackground, RowBlock, RowGap, RowWidget},
    component::input::EditorState,
    div,
    prelude::FluentBuilder,
};

use super::helpers::{add_comment_button, compact_bar, decide_at_caret, hunk_bar, washes};
pub use super::resolve::Resolve;
use super::types::{DecideHandler, Decision, RowHandler};
use crate::{code_editor::CodeEditor, theme::ActiveTheme};

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
        Self {
            id: id.into(),
            removed,
            added,
        }
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

/// The review as it stood before and after each decision, so an undo or a redo that brings a
/// decision's text back brings its hunks back too. Undo restores text only; without this a hunk the
/// user took back would come back as plain code with no bands and no bar.
#[derive(Default)]
pub struct DecisionHistory {
    /// `(before, after)` for each decision, oldest first; each side is the text and its hunks.
    pub(super) decisions: Vec<[(String, Vec<InlineHunk>); 2]>,
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
        self.decisions
            .iter()
            .rev()
            .flatten()
            .find(|(known, _)| known == text)
            .map(|(_, hunks)| hunks.clone())
    }
}

/// The agent's edits, laid over the file the user is editing.
///
/// The editor stays writable. A review does not own the buffer, so the user can fix their own typo in
/// the middle of a hunk and then accept it.
#[derive(IntoElement)]
pub struct InlineReview {
    pub(super) id: ElementId,
    pub(super) state: Entity<EditorState>,
    pub(super) hunks: Vec<InlineHunk>,
    pub(super) current: Option<SharedString>,
    pub(super) height: Option<Pixels>,
    pub(super) on_decide: Option<DecideHandler>,
    resolving: Vec<Resolve>,
    on_resolved: Option<DecideHandler>,
    row_blocks: Vec<RowBlock>,
    on_add_comment: Option<RowHandler>,
    on_card: bool,
    pub(super) decisions: bool,
    read_only: bool,
    pub(super) fill: bool,
}

impl InlineReview {
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<EditorState>,
        hunks: Vec<InlineHunk>,
    ) -> Self {
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
    pub fn on_resolved(
        mut self,
        f: impl Fn(&SharedString, Decision, &mut Window, &mut App) + 'static,
    ) -> Self {
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
                Some(RowBackground {
                    rows: hunk.closing(decision),
                    color: fill.opacity(1. - fade),
                    marker: None,
                })
            })
            .collect();
        // The editor paints the washes and places the bars from its own layout, so they sit exactly
        // on their rows whatever its padding, row height, wrapping or scroll.
        let backgrounds = self
            .hunks
            .iter()
            .flat_map(|hunk| {
                let fade = fades.get(&hunk.id).map_or(1., |(fade, _)| *fade);
                washes(std::slice::from_ref(hunk))
                    .into_iter()
                    .map(move |(rows, added)| (rows, added, fade))
            })
            .map(|(rows, added, fade)| RowBackground {
                rows,
                color: theme.diff_line(added).opacity(fade),
                marker: Some(if added { theme.success } else { theme.danger }.opacity(fade)),
            })
            .collect();
        let bar_fill = theme.card_strong;
        // The review's own width in the last frame; the first frame draws the full bar.
        let width = window.use_keyed_state(
            ElementId::NamedChild(Arc::new(self.id.clone()), "width".into()),
            cx,
            |_, _| f32::MAX,
        );
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
                        hunk_bar(
                            hunk.clone(),
                            is_current,
                            compact,
                            bar_fill,
                            on_decide.clone(),
                        )
                        .into_any_element()
                    }),
                }
            })
            .collect();
        let add = self
            .on_add_comment
            .clone()
            .map(|add| -> input::GutterWidget {
                let (fill, ink) = (theme.primary, theme.primary_foreground);
                Rc::new(move |row, _, _| {
                    add_comment_button(row, fill, ink, add.clone()).into_any_element()
                })
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
        let open: Vec<InlineHunk> = self
            .hunks
            .iter()
            .filter(|h| self.decisions && !deciding(h))
            .cloned()
            .collect();
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
