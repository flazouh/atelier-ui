use std::sync::Arc;

use gpui_kit::{App, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, Styled, div};

use crate::scale::px;
use crate::{
    ClickHandler,
    button::Button,
    file_diff::{FileDiff, FileDiffStatus},
    motion::{Curve, ease},
    theme::{Theme, radius},
    tool_preview::ToolPreview,
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::structs::ApprovalMotion;
use super::types::{ParamValue, ToolApprovalStatus};

pub(super) fn child(id: &ElementId, name: &'static str) -> ElementId {
    ElementId::NamedChild(Arc::new(id.clone()), name.into())
}

pub(super) fn wire(button: Button, handler: Option<ClickHandler>) -> Button {
    match handler {
        Some(h) => button.on_click(move |e, w, cx| h(e, w, cx)),
        None => button,
    }
}

/// The words in the head of the card: the title, the tool's own name line, and the description. A preview that
/// names a file makes the head one line, the tool's word and the file ("Edit NOTES.md"), because the diff below
/// repeats the path; the tool line goes when it says what the title says, and so does a description that only
/// gives the path. A command preview shows the command whole, so a description that only repeats the start of it
/// goes. With no preview, the path line stays.
pub fn head_words(title: &str, tool: &str, description: Option<&str>, preview_path: Option<&str>, preview_command: Option<&str>) -> (String, Option<String>, Option<String>) {
    let mut title = title.to_string();
    if let Some(path) = preview_path
        && title == tool
    {
        title = format!("{tool} {path}");
    }
    // The tool line goes when the title already says it: the same words, or the tool word and then its target.
    let tool_line = (tool != title && !title.strip_prefix(tool).is_some_and(|rest| rest.starts_with(' ') && preview_path.is_some_and(|p| rest.trim() == p))).then(|| tool.to_string());
    // A command block shows the command whole, so a description that only starts it (cut at an ellipsis) goes.
    let repeats_command = |d: &str| preview_command.is_some_and(|c| c.trim().starts_with(d.trim_end_matches(['…', '.']).trim()));
    let description = description.filter(|d| preview_path.is_none_or(|path| *d != path) && !repeats_command(d)).map(str::to_string);
    (title, tool_line, description)
}

/// The preview: a diff under its path, or the command in a mono block.
pub(super) fn preview_view(id: &ElementId, preview: &ToolPreview, theme: &Theme) -> impl IntoElement {
    let body = match preview {
        ToolPreview::Command { text } => div()
            .flex()
            .gap(px(8.))
            .p(px(12.))
            .rounded(radius::xl())
            .bg(theme.card_strong)
            .font_family(MONO_FONT_FAMILY)
            .text_size(TextSize::Xs.font_size())
            .text_color(theme.foreground.opacity(0.85))
            .child(div().flex_none().text_color(theme.muted_foreground).child("$"))
            .child(div().min_w_0().child(text.clone()))
            .into_any_element(),
        ToolPreview::Edits { path, .. } | ToolPreview::Written { path, .. } => {
            FileDiff::new(child(id, "preview"), path.clone(), preview.rows())
                .status(FileDiffStatus::Complete)
                .default_open(true)
                .collapse_on_complete(false)
                .into_any_element()
        }
    };
    div().debug_selector(|| "approval-preview".into()).mx(px(16.)).mb(px(12.)).child(body)
}

pub(super) fn param_value(value: ParamValue, theme: &Theme) -> impl IntoElement {
    let mono = theme.foreground.opacity(0.85);
    let base = div().min_w_0().font_family(MONO_FONT_FAMILY).text_color(mono);
    match value {
        ParamValue::Text(text) => base.child(text),
        ParamValue::Code(code) => {
            base.child(div().rounded(radius::lg()).bg(theme.card.opacity(0.3)).px(px(10.)).py(px(8.)).child(code))
        }
    }
}

/// Closes the details when the status leaves `Pending`, and fades the action row with it, like beui's
/// `useEffect` plus `AnimatePresence`.
pub(super) fn follow_status(motion: &Entity<ApprovalMotion>, status: ToolApprovalStatus, reduce: bool, cx: &mut App) {
    motion.update(cx, |m, _| {
        if m.status != status {
            let was_pending = m.status == ToolApprovalStatus::Pending;
            m.status = status;
            let pending = status == ToolApprovalStatus::Pending;
            if pending != was_pending {
                // `reduce` alone decides whether this jumps; the duration only matters when it doesn't.
                m.actions.animate(if pending { 1. } else { 0. }, Curve::Ease(0.22, ease::OUT), 0., reduce);
            }
            if was_pending && !pending {
                m.disclosure.set_open(false, reduce);
            }
        }
    });
}
