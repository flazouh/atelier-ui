//! One session under its project in the sidebar: the agent's mark, the title, and how it stands. The
//! status is data; the row has no idea what an agent is.
//!
//! - Working: the agent's mark animates and the title is ink.
//! - Needs you: the warning tone and the words ("Needs approval").
//! - Finished and not yet seen: a small amber dot, the title in ink. Its changes are ready to look at.
//! - Seen and idle: the still mark, a muted title, the time since it last did anything.
//! - Failed: the danger tone and "Stopped: <reason>".
//!
//! A change of status rolls the old mark up out of its box and the new one up into it ([`crate::roll`]); with
//! Reduce Motion the mark changes at once.
use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    agent_look::AgentLook,
    icon::{Icon, IconName},
    roll::{Kind as RollKind, Roll},
    session_status::{Mark, SessionStatus},
    sidebar_model::{SessionData, since},
    theme::{ActiveTheme, Theme, radius},
    tooltip::Tooltip,
    typography::TextSize,
};

/// Every sidebar row is this tall, so the list is virtual.
pub const ROW_HEIGHT: f32 = 32.;
/// The box a mark is drawn in.
pub const MARK_BOX: f32 = 16.;

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

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct SessionRow {
    id: ElementId,
    data: SessionData,
    now: u64,
    selected: bool,
    open: bool,
    on_open: Option<Handler>,
    /// The ⋯ button: pressed, and the menu it opens while `more_open`.
    on_more: Option<Handler>,
    more_open: bool,
    more_menu: Option<AnyElement>,
    /// The project's badge and name, on a row of the priority list, where no project heading says which it is.
    project: Option<(crate::sidebar_model::Badge, SharedString)>,
    /// The row sits at the list's edge, with no project heading above to indent under.
    flush: bool,
    show_time: bool,
    show_icon: bool,
}

impl SessionRow {
    /// `now` is the time to count "2m" from, in seconds since the Unix epoch.
    pub fn new(id: impl Into<ElementId>, data: SessionData, now: u64) -> Self {
        Self { id: id.into(), data, now, selected: false, open: false, on_open: None, on_more: None, more_open: false, more_menu: None, project: None, flush: false, show_time: true, show_icon: true }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// This is the session the reader has open: a bar at the row's edge and the title in medium ink, so the row can be
    /// found among the others whatever its status.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn on_open(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }

    /// Shows the project's badge on the row (its name in the tooltip).
    pub fn project(mut self, badge: crate::sidebar_model::Badge, name: impl Into<SharedString>) -> Self {
        self.project = Some((badge, name.into()));
        self
    }

    /// What the row shows and where it sits, from the sidebar's layout: the one place that decides it.
    pub fn layout(mut self, layout: &crate::sidebar_layout::SidebarLayout) -> Self {
        self.flush = layout.rows_flush();
        self.show_time = layout.show_time;
        self.show_icon = layout.show_agent_icon;
        self
    }

    /// A ⋯ button at the row's end, shown while the pointer is on the row (and while its menu is open). `menu` is the
    /// menu hung under it.
    pub fn more(mut self, open: bool, press: impl Fn(&mut Window, &mut App) + 'static, menu: Option<AnyElement>) -> Self {
        self.on_more = Some(Rc::new(press));
        self.more_open = open;
        self.more_menu = menu;
        self
    }
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

impl RenderOnce for SessionRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let data = self.data;
        let mark = agent_icon((self.id.clone(), "mark"), &data.look, &data.status, &theme, self.show_icon);
        let (words, tone) = trailing(&data.status, self.now, data.active_at, &theme);
        let ink = if data.status.title_is_ink() || self.open { theme.foreground } else { theme.muted_foreground };
        let words_of_status = data.status.words();
        let mark_id = self.id.clone();
        // Hidden until the pointer is on the row, then it takes the place of the time; a press does not open the session.
        let more_open = self.more_open;
        let has_more = self.on_more.is_some();
        let more = self.on_more.map(|press| {
            div()
                .absolute()
                .right(px(4.))
                .top(px((ROW_HEIGHT - 22.) / 2.))
                .when(!more_open, |d| d.invisible().group_hover("session-row", |s| s.visible()))
                .child(
                    div().relative().child(
                        crate::button::Button::new((self.id.clone(), "more-button"))
                            .debug_name("session-more")
                            .icon(IconName::MoreHoriz)
                            .variant(crate::button::ButtonVariant::Ghost)
                            .size(crate::button::ButtonSize::IconSm)
                            .tooltip("More")
                            .open(more_open)
                            .on_click(move |_, window, cx| {
                                cx.stop_propagation();
                                press(window, cx)
                            }),
                    )
                    .children(self.more_menu),
                )
        });
        div()
            .id(self.id)
            .group("session-row")
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(8.))
            .h(px(ROW_HEIGHT))
            .pl(px(if self.flush { 14. } else { 26. }))
            .pr(px(10.))
            .w_full()
            .rounded(radius::md())
            .cursor_pointer()
            .text_size(TextSize::Sm.font_size())
            // The keyboard's row is marked only while the keyboard is the reader's hand; a press selects nothing.
            .when(self.selected && window.last_input_was_keyboard(), |d| d.bg(theme.card_strong).child(crate::focus::row_ring(&theme, theme.background, radius::md())))
            // The one open in front (single view only) is soft on its card.
            .when(self.open, |d| d.bg(theme.card_strong.opacity(0.6)))
            .hover(|s| s.bg(theme.card_strong.opacity(0.6)))
            .when_some(self.on_open, |d, open| d.on_click(move |_, window, cx| open(window, cx)))
            .when(self.open, |d| d.child(div().absolute().left(px(12.)).top(px((ROW_HEIGHT - 16.) / 2.)).w(px(3.)).h(px(16.)).rounded_full().bg(theme.foreground).debug_selector(|| "session-row-open".into())))
            .child(div().id((mark_id.clone(), "status")).tooltip(Tooltip::text(words_of_status)).child(mark))
            .child({
                let title = data.title.clone();
                div()
                    .debug_selector(move || format!("row-title:{title}"))
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_color(ink)
                    .when(self.open, |t| t.font_weight(FontWeight::MEDIUM))
                    .child(data.title)
            })
            .children(self.project.map(|(badge, name)| {
                div()
                    .id((mark_id.clone(), "project"))
                    .debug_selector(|| "row-project".into())
                    .flex_none()
                    .tooltip(Tooltip::text(name))
                    .child(crate::project_badge::ProjectBadge::new(badge.label, badge.color).icon(badge.icon))
            }))
            // The time makes room for the ⋯ while the pointer is on the row; short words line up in a column.
            .when(self.show_time || !matches!(data.status, SessionStatus::Idle | SessionStatus::Working | SessionStatus::Finished), |d| d.child(
                div()
                    .flex_none()
                    .min_w(px(26.))
                    .max_w(px(150.))
                    .truncate()
                    .text_right()
                    .text_size(TextSize::Xs.font_size())
                    .text_color(tone)
                    .when(has_more, |d| d.group_hover("session-row", |s| s.invisible()))
                    .child(words),
            ))
            .children(more)
    }
}

#[cfg(test)]
mod tests;
