//! The editable code surface: gpui-base's code editor, skinned for atelier.
//!
//! gpui-base owns the hard parts, so this file does not reimplement any of them: editing, selection,
//! undo, IME, search, folding, line numbers, and tree-sitter highlighting all come from
//! [`gpui_kit::component::input::EditorState`]. atelier supplies the look and the language list.
//!
//! - Geist Mono at `text-xs`, 20px rows, the gutter in `muted_foreground` at 40%, the current line
//!   washed with `card`.
//! - Token colors come from the theme in force (`assets/themes`); atelier's are the muted palette: keywords take
//!   `info`, strings `success`, numbers and escapes `warning`, comments muted, types the ramp's light
//!   tone. Nothing here uses Tailwind's chroma.
//! - Diagnostics underline their range in `danger` for an error and `warning` for a warning. The caller
//!   pushes them; [`crate::lsp`] is what fills them in.
//! - `read_only` is for a file with a hunk review open, so the two never fight over one buffer.
//! - Four-space indents, and the line commands in [`commands`] on Zed's keys: toggle comment, move
//!   line, duplicate line, delete line, a Home that stops at the indent, and copy, cut or paste of the
//!   whole line when nothing is selected.

use gpui_kit::{
    App, AppContext, ClipboardItem, Entity, InteractiveElement, IntoElement, KeyBinding, ParentElement, Pixels,
    RenderOnce, SharedString, Styled, Window,
    base::input::{self as input, DiagnosticSeverity, InputEditorStyle, TabSize},
    component::{
        highlighter::HighlightTheme,
        input::{Editor, EditorState},
    },
    div,
    prelude::FluentBuilder,
    
};
use crate::scale::px;
use std::sync::Arc;

pub mod commands;

use commands::Edit;

gpui_kit::actions!(
    code_editor,
    [
        /// Comments out the selected lines, or uncomments them.
        ToggleComment,
        /// Swaps the selected lines with the line above.
        MoveLineUp,
        /// Swaps the selected lines with the line below.
        MoveLineDown,
        /// Copies the selected lines below themselves.
        DuplicateLine,
        /// Removes the selected lines.
        DeleteLine,
        /// Home that stops at the indent first.
        SmartHome,
        /// Copy, or the whole line when nothing is selected.
        CopyLine,
        /// Cut, or the whole line when nothing is selected.
        CutLine,
        /// Paste, and a whole line copied that way goes above the caret's line.
        PasteLine,
        /// Lists every use of the symbol at the caret. The editor only names the key; whoever owns the
        /// language server answers it.
        FindReferences,
    ]
);

/// The key context around every [`CodeEditor`], so these keys never reach a one-line input.
const CONTEXT: &str = "CodeEditor > Input";

/// Binds the line commands. [`crate::init`] calls it after gpui-kit's own keys, so ours win where
/// both bind a key.
pub(crate) fn bind_keys(cx: &mut App) {
    let mac = cfg!(target_os = "macos");
    let secondary = |key: &str| if mac { format!("cmd-{key}") } else { format!("ctrl-{key}") };
    cx.bind_keys([
        KeyBinding::new(&secondary("/"), ToggleComment, Some(CONTEXT)),
        KeyBinding::new("alt-up", MoveLineUp, Some(CONTEXT)),
        KeyBinding::new("alt-down", MoveLineDown, Some(CONTEXT)),
        KeyBinding::new(&secondary("shift-d"), DuplicateLine, Some(CONTEXT)),
        KeyBinding::new(&secondary("shift-k"), DeleteLine, Some(CONTEXT)),
        KeyBinding::new("home", SmartHome, Some(CONTEXT)),
        KeyBinding::new(&secondary("c"), CopyLine, Some(CONTEXT)),
        KeyBinding::new(&secondary("x"), CutLine, Some(CONTEXT)),
        KeyBinding::new(&secondary("v"), PasteLine, Some(CONTEXT)),
        KeyBinding::new("shift-f12", FindReferences, Some(CONTEXT)),
    ]);
}

/// Marks a clipboard entry as one whole line, so a paste knows to put it above the caret's line.
const WHOLE_LINE: &str = "atelier.whole-line";

/// Applies `edit` as one undo step and puts every selection back. `None` changes nothing.
fn apply(state: &Entity<EditorState>, edit: Option<Edit>, window: &mut Window, cx: &mut App) {
    let Some(edit) = edit else { return };
    state.update(cx, |state, cx| {
        state.set_selected_range(edit.range, cx);
        state.replace(edit.text, window, cx);
        state.set_selected_ranges(&edit.selections, cx);
    });
}

/// A listener for action `A` that runs `run` on the editor's state.
fn on<A: gpui_kit::Action>(
    state: &Entity<EditorState>,
    run: fn(&Entity<EditorState>, &mut Window, &mut App),
) -> impl Fn(&A, &mut Window, &mut App) + 'static {
    let state = state.clone();
    move |_, window, cx| run(&state, window, cx)
}

/// The buffer and every selection, the input to every command.
fn snapshot(state: &Entity<EditorState>, cx: &App) -> (String, Vec<std::ops::Range<usize>>) {
    let state = state.read(cx);
    (state.value().to_string(), state.selected_ranges())
}

/// Whether no selection holds any text, which is when copy and cut take whole lines.
fn all_bare(selections: &[std::ops::Range<usize>]) -> bool {
    selections.iter().all(|s| s.is_empty())
}

use crate::{
    theme::{ActiveTheme, Appearance, StatusTone, Theme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

/// Row height, matching the hunk review so a diff and its file line up.
pub const ROW_HEIGHT: f32 = 20.;

pub use crate::syntax::{LANGUAGES, language_for};

/// atelier's syntax colours in one appearance, for code drawn outside a window, such as a bench.
pub fn syntax_theme(appearance: Appearance) -> Arc<HighlightTheme> {
    crate::themes::atelier(appearance).syntax.clone()
}

/// Installs `theme`'s token colours. [`crate::theme::set_theme`] calls this, so a switch re-reads them.
pub fn install_syntax_theme(theme: &Theme, cx: &mut App) {
    gpui_kit::component::Theme::global_mut(cx).highlight_theme = theme.syntax.clone();
}

/// The fill an editor sits on: the page, or a card, and the current line's wash on it.
pub(crate) fn surface(theme: &Theme, on_card: bool) -> (gpui_kit::Hsla, gpui_kit::Hsla) {
    if on_card { (theme.card, theme.card_strong) } else { (theme.background, theme.card) }
}

/// The editor's own colors, from the theme.
fn editor_style(theme: &Theme, appearance: Appearance, on_card: bool) -> InputEditorStyle {
    let muted = theme.muted_foreground;
    let syntax = theme.syntax.clone();
    let (fill, active_line) = surface(theme, on_card);
    let mut style = InputEditorStyle {
        foreground: theme.foreground,
        muted_foreground: muted,
        background: fill,
        // gpui-base draws only the indent guides in this colour; the editor has no frame. A faint
        // muted line, as in Zed, so a guide never cuts a dark stripe through a row wash.
        border: muted.opacity(0.14),
        // Not `primary`: that is now near-solid ink, and a 35% wash of it buries the text under
        // the caret. Amber at a low alpha keeps every glyph legible and reads as a highlight.
        // Cream is lighter than the dark page, so light needs more amber to read as a selection
        // over the current-line wash.
        selection: theme.accent.opacity(match appearance {
            Appearance::Dark => 0.20,
            Appearance::Light => 0.32,
        }),
        caret: theme.foreground,
        diagnostics: Default::default(),
        highlight_styles: syntax,
        editor_invisible: Some(muted.opacity(0.25)),
        editor_active_line: Some(active_line),
        editor_gutter_background: Some(fill),
        fold_icon_renderer: None,
    };
    style.diagnostics.error = theme.danger;
    style.diagnostics.warning = theme.warning;
    style.diagnostics.info = theme.info;
    style.diagnostics.hint = muted;
    style
}

/// What a diagnostic's severity means for the gutter mark, so an error reads at a glance and a hint
/// stays quiet. It uses the same ramp as the tool rows.
pub fn severity_tone(severity: DiagnosticSeverity) -> StatusTone {
    match severity {
        DiagnosticSeverity::Error => StatusTone::Failed,
        DiagnosticSeverity::Warning => StatusTone::Pending,
        DiagnosticSeverity::Info => StatusTone::Done,
        DiagnosticSeverity::Hint => StatusTone::Cancelled,
    }
}

/// Replaces what the editor shows as wrong with `diagnostics`, straight from a language server.
/// gpui-base converts each one, so nothing is reinterpreted on the way in. An empty list clears them,
/// which is what a server sends when a file becomes clean.
pub fn set_diagnostics(state: &Entity<EditorState>, diagnostics: Vec<lsp_types::Diagnostic>, cx: &mut App) {
    state.update(cx, |state, cx| {
        let text = state.text().clone();
        // A single-line input has no diagnostics to hold; there is nothing to show them on.
        if let Some(set) = state.diagnostics_mut() {
            set.reset(&text);
            for diagnostic in diagnostics {
                set.push(diagnostic);
            }
        }
        cx.notify();
    });
}

/// A file open for reading and writing.
#[derive(IntoElement)]
pub struct CodeEditor {
    state: Entity<EditorState>,
    height: Option<Pixels>,
    fill: bool,
    read_only: bool,
    on_card: bool,
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
                let homes = commands::smart_home(&text, &selections);
                state.update(cx, |state, cx| state.set_selected_ranges(&homes, cx));
            }))
            .on_action(on::<CopyLine>(&state, |state, window, cx| {
                let (text, selections) = snapshot(state, cx);
                if all_bare(&selections) {
                    let (_, lines) = commands::whole_lines(&text, &selections);
                    cx.write_to_clipboard(ClipboardItem::new_string_with_metadata(lines, WHOLE_LINE.into()));
                } else {
                    window.dispatch_action(Box::new(input::Copy), cx);
                }
            }))
            .when(editable, |d| {
                d.on_action(on::<ToggleComment>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    let language = state.read(cx).language_name();
                    if let Some(prefix) = commands::comment_prefix(&language) {
                        apply(state, commands::toggle_comment(&text, &selections, prefix), window, cx);
                    }
                }))
                .on_action(on::<MoveLineUp>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    apply(state, commands::move_lines(&text, &selections, true), window, cx);
                }))
                .on_action(on::<MoveLineDown>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    apply(state, commands::move_lines(&text, &selections, false), window, cx);
                }))
                .on_action(on::<DuplicateLine>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    apply(state, commands::duplicate_lines(&text, &selections), window, cx);
                }))
                .on_action(on::<DeleteLine>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    apply(state, commands::delete_lines(&text, &selections), window, cx);
                }))
                .on_action(on::<CutLine>(&state, |state, window, cx| {
                    let (text, selections) = snapshot(state, cx);
                    if all_bare(&selections) {
                        let (_, lines) = commands::whole_lines(&text, &selections);
                        cx.write_to_clipboard(ClipboardItem::new_string_with_metadata(lines, WHOLE_LINE.into()));
                        apply(state, commands::delete_lines(&text, &selections), window, cx);
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
                            apply(state, commands::whole_line_paste(&text, &selections, &line), window, cx)
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

#[cfg(test)]
mod tests;
