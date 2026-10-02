use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    agent_look::AgentLook,
    icon::{Icon, IconName},
    roll::{Kind as RollKind, Roll},
    session_status::{Mark, SessionStatus},
    sidebar_model::since,
    theme::{ActiveTheme, Theme},
};
use super::types::MARK_BOX;

/// The mark for a status: the agent's own, or a tone, or the amber dot. When the status changes the old mark rolls
/// up out of its box and the new one rolls up into it ([`Roll`]); the roll is keyed by `id`, so it belongs to one row
/// or tab.
pub fn status_mark(id: impl Into<ElementId>, look: &AgentLook, status: &SessionStatus, _window: &mut Window, cx: &mut App) -> AnyElement {
    let id = id.into();
    let theme = cx.theme().clone();
    let look = look.clone();
    let sprite_id = id.clone();
    let drawn = move |mark: &Mark| -> AnyElement {
        let drawn = match mark {
            Mark::AgentWorking => look.mark.sprite(sprite_id.clone(), look.mark.working).size(px(14.)).into_any_element(),
            Mark::Idle => div().size(px(8.)).rounded_full().border(px(1.5)).border_color(theme.muted_foreground.opacity(0.7)).into_any_element(),
            Mark::Warning => Icon::new(IconName::PriorityHigh).size(px(14.)).color(theme.warning).into_any_element(),
            Mark::Danger => Icon::new(IconName::Error).size(px(14.)).color(theme.danger).into_any_element(),
            Mark::AmberDot => div().size(px(8.)).rounded_full().bg(theme.accent).into_any_element(),
        };
        div().flex().flex_none().size(px(MARK_BOX)).items_center().justify_center().child(drawn).into_any_element()
    };
    Roll::new((id, "roll"), status.mark(), RollKind::Icon, px(MARK_BOX), drawn).into_any_element()
}

/// The agent's mark for a row: still in the muted tone, and playing in the agent's own colour while it works. A dot in
/// its corner says what the reader owes or has: amber for a finished turn not yet seen, the warning tone for a
/// question or an approval, the danger tone for a stop.
pub fn agent_icon(id: impl Into<ElementId>, look: &AgentLook, status: &SessionStatus, theme: &Theme, with_icon: bool) -> AnyElement {
    let working = matches!(status, SessionStatus::Working);
    let color = if working { look.mark.color } else if status.title_is_ink() { theme.foreground.opacity(0.8) } else { theme.muted_foreground.opacity(0.7) };
    let icon = crate::sprite::Sprite::new(id, look.mark.working, color).rest(look.mark.working).still_frame(look.mark.icon_frame).size(px(14.)).playing(working);
    let dot = match status {
        SessionStatus::Finished => Some(theme.accent),
        SessionStatus::NeedsYou(_) => Some(theme.warning),
        SessionStatus::Failed(_) => Some(theme.danger),
        SessionStatus::Working | SessionStatus::Idle => None,
    };
    div()
        .relative()
        .flex()
        .flex_none()
        .size(px(MARK_BOX))
        .items_center()
        .justify_center()
        .when(with_icon, |d| d.child(icon))
        .children(dot.map(|dot| div().absolute().right(px(-1.)).bottom(px(-1.)).size(px(7.)).rounded_full().bg(dot).debug_selector(|| "session-dot".into())))
        .into_any_element()
}

/// The words on the right of a row: what is owed, or the time since it last did anything.
pub fn trailing(status: &SessionStatus, now: u64, active_at: u64, theme: &Theme) -> (SharedString, gpui_kit::Hsla) {
    match status {
        SessionStatus::NeedsYou(_) => (status.words(), theme.warning),
        SessionStatus::Failed(_) => (status.words(), theme.danger),
        _ => (since(now, active_at).into(), theme.muted_foreground.opacity(0.8)),
    }
}
