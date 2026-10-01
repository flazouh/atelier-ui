use super::{
    CopyLine, CutLine, DeleteLine, DuplicateLine, MoveLineDown, MoveLineUp, PasteLine,
    SmartHome, ToggleComment,
};

use gpui_kit::{
    App,
    AppContext,
    ClipboardItem,
    Entity,
    InteractiveElement,
    IntoElement,
    ParentElement,
    Pixels,
    RenderOnce,
    SharedString,
    Styled,
    Window,
    base::input::{self as input, TabSize},
    component::{input::{Editor, EditorState}},
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
pub use crate::syntax::language_for;
use super::types::{ROW_HEIGHT, WHOLE_LINE};
use super::helpers::{all_bare, apply, editor_style, on, snapshot, surface};

/// A file open for reading and writing.
#[derive(IntoElement)]
pub struct CodeEditor {
    pub(super) state: Entity<EditorState>,
    pub(super) height: Option<Pixels>,
    pub(super) fill: bool,
    pub(super) read_only: bool,
    pub(super) on_card: bool,
}

impl CodeEditor {
    /// The caller owns the state, so the buffer outlives one frame. Build it with
    /// [`CodeEditor::state`] so the language and the row height are set the same way every time.
    pub fn new(state: &Entity<EditorState>) -> Self {
        Self { state: state.clone(), height: None, fill: false, read_only: false, on_card: false }
    }

    /// A state for `path`, with its language picked by extension and `text` as the buffer.
    pub fn state(
        path: &str,
        text: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<EditorState> {
        let language = language_for(path);
        let text = text.into();
        cx.new(|cx| {
            // Four spaces, as rustfmt, prettier and Zed indent. gpui-base defaults to two.
            let mut state = EditorState::new(window, cx)
                .line_number(true)
                .tab_size(TabSize { tab_size: 4, hard_tabs: false });
            if let Some(language) = language {
                state = state.language(language);
            }
            state.set_value(text, window, cx);
            state
        })
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    /// Takes its parent's whole height, as an editor pane does, rather than a height of its own.
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// Stops edits while a hunk review owns the same file.
    /// Sits on a card instead of the page: the card's fill, with the current line one step up.
    pub fn on_card(mut self, on_card: bool) -> Self {
        self.on_card = on_card;
        self
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
}

impl RenderOnce for CodeEditor {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let appearance = theme.appearance;
        // Pinned, not set: gpui-component's Input sets its theme's style on every render, after this
        // one, and would otherwise replace ours (vendor/gpui-base/PATCHES.md, patch 3).
        let on_card = self.on_card;
        self.state.update(cx, |state, _| state.pin_editor_style(Some(editor_style(&theme, appearance, on_card))));

        let state = self.state.clone();
        let editable = !self.read_only;
        div()
            .key_context("CodeEditor")
            .on_action(on::<SmartHome>(&state, |state, _, cx| {
                let (text, selections) = snapshot(state, cx);
                let homes = super::commands::smart_home(&text, &selections);
                state.update(cx, |state, cx| state.set_selected_ranges(&homes, cx));
            }))
            .on_action(on::<CopyLine>(&state, |state, window, cx| {
                let (text, selections) = snapshot(state, cx);
                if all_bare(&selections) {
                    let (_, lines) = super::commands::whole_lines(&text, &selections);
                    cx.write_to_clipboard(ClipboardItem::new_string_with_metadata(lines, WHOLE_LINE.into()));
                } else {
                    window.dispatch_action(Box::new(input::Copy), cx);
                }
            }))
            .when(editable, |d| {
                d.on_action(on::<ToggleComment>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    let language = state.read(cx).language_name();
                    if let Some(prefix) = super::commands::comment_prefix(&language) {
                        apply(state, super::commands::toggle_comment(&text, &selections, prefix), window, cx);
                    }
                }))
                .on_action(on::<MoveLineUp>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    apply(state, super::commands::move_lines(&text, &selections, true), window, cx);
                }))
                .on_action(on::<MoveLineDown>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    apply(state, super::commands::move_lines(&text, &selections, false), window, cx);
                }))
                .on_action(on::<DuplicateLine>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    apply(state, super::commands::duplicate_lines(&text, &selections), window, cx);
                }))
                .on_action(on::<DeleteLine>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    apply(state, super::commands::delete_lines(&text, &selections), window, cx);
                }))
                .on_action(on::<CutLine>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    if all_bare(&selections) {
                        let (_, lines) = super::commands::whole_lines(&text, &selections);
                        cx.write_to_clipboard(ClipboardItem::new_string_with_metadata(lines, WHOLE_LINE.into()));
                        apply(state, super::commands::delete_lines(&text, &selections), window, cx);
                    } else {
                        window.dispatch_action(Box::new(input::Cut), cx);
                    }
                }))
                .on_action(on::<PasteLine>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    let clipboard = cx.read_from_clipboard();
                    let whole = clipboard.as_ref().and_then(|c| c.metadata()).is_some_and(|m| m == WHOLE_LINE);
                    // A copied whole line goes above each bare caret's line; anything else pastes as
                    // usual, including gpui-base's one-line-per-cursor paste.
                    match clipboard.and_then(|c| c.text()) {
                        Some(line) if whole && all_bare(&selections) => {
                            apply(state, super::commands::whole_line_paste(&text, &selections, &line), window, cx)
                        }
                        _ => window.dispatch_action(Box::new(input::Paste), cx),
                    }
                }))
            })
            .flex()
            .flex_col()
            .w_full()
            .when(self.fill, |d| d.h_full())
            .overflow_hidden()
            .rounded(radius::xl())
            .bg(surface(&theme, on_card).0)
            .font_family(MONO_FONT_FAMILY)
            .text_size(TextSize::Xs.font_size())
            .line_height(px(ROW_HEIGHT))
            .child(
                Editor::new(&self.state)
                    .appearance(false)
                    .bordered(false)
                    .readonly(self.read_only)
                    .when_some(self.height, |e, h| e.h(h))
                    .when(self.fill, |e| e.h_full()),
            )
    }
}
