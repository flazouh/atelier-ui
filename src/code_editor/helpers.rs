use super::{
    CopyLine, CutLine, DeleteLine, DuplicateLine, FindReferences, MoveLineDown, MoveLineUp,
    PasteLine, SmartHome, ToggleComment,
};

use std::sync::Arc;

use gpui_kit::{
    App,
    Entity,
    KeyBinding,
    Window,
    base::input::{DiagnosticSeverity, InputEditorStyle},
    component::{highlighter::HighlightTheme, input::{EditorState}},
};

use super::commands::Edit;
use crate::{
    theme::{Appearance, StatusTone, Theme},
    };
use super::types::CONTEXT;

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

/// Applies `edit` as one undo step and puts every selection back. `None` changes nothing.
pub(super) fn apply(state: &Entity<EditorState>, edit: Option<Edit>, window: &mut Window, cx: &mut App) {
    let Some(edit) = edit else { return };
    state.update(cx, |state, cx| {
        state.set_selected_range(edit.range, cx);
        state.replace(edit.text, window, cx);
        state.set_selected_ranges(&edit.selections, cx);
    });
}

/// A listener for action `A` that runs `run` on the editor's state.
pub(super) fn on<A: gpui_kit::Action>(
    state: &Entity<EditorState>,
    run: fn(&Entity<EditorState>, &mut Window, &mut App),
) -> impl Fn(&A, &mut Window, &mut App) + 'static {
    let state = state.clone();
    move |_, window, cx| run(&state, window, cx)
}

/// The buffer and every selection, the input to every command.
pub(super) fn snapshot(state: &Entity<EditorState>, cx: &App) -> (String, Vec<std::ops::Range<usize>>) {
    let state = state.read(cx);
    (state.value().to_string(), state.selected_ranges())
}

/// Whether no selection holds any text, which is when copy and cut take whole lines.
pub(super) fn all_bare(selections: &[std::ops::Range<usize>]) -> bool {
    selections.iter().all(|s| s.is_empty())
}

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
pub(super) fn editor_style(theme: &Theme, appearance: Appearance, on_card: bool) -> InputEditorStyle {
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
