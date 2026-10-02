use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    icon::IconName,
    session_status::SessionStatus,
    sidebar_model::SessionData,
    theme::{ActiveTheme, radius},
    tooltip::Tooltip,
    typography::TextSize,
};
use super::types::{Handler, ROW_HEIGHT};
use super::helpers::{agent_icon, trailing};

#[derive(IntoElement)]
pub struct SessionRow {
    pub(super) id: ElementId,
    pub(super) data: SessionData,
    pub(super) now: u64,
    selected: bool,
    pub(super) open: bool,
    on_open: Option<Handler>,
    /// The ⋯ button: pressed, and the menu it opens while `more_open`.
    on_more: Option<Handler>,
    more_open: bool,
    more_menu: Option<AnyElement>,
    /// The archive button before the ⋯: whether the session is archived now, and the press.
    on_archive: Option<(bool, Handler)>,
    /// The project's badge and name, on a row of the priority list, where no project heading says which it is.
    pub(super) project: Option<(crate::sidebar_model::Badge, SharedString)>,
    /// The row sits at the list's edge, with no project heading above to indent under.
    flush: bool,
    show_time: bool,
    show_icon: bool,
}

impl SessionRow {
    /// `now` is the time to count "2m" from, in seconds since the Unix epoch.
    pub fn new(id: impl Into<ElementId>, data: SessionData, now: u64) -> Self {
        Self { id: id.into(), data, now, selected: false, open: false, on_open: None, on_more: None, more_open: false, more_menu: None, on_archive: None, project: None, flush: false, show_time: true, show_icon: true }
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

    /// An archive button before the ⋯, shown while the pointer is on the row; an archived session gets unarchive.
    pub fn archive(mut self, archived: bool, press: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_archive = Some((archived, Rc::new(press)));
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

impl RenderOnce for SessionRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let data = self.data;
        let mark = agent_icon((self.id.clone(), "mark"), &data.look, &data.status, &theme, self.show_icon);
        let (words, tone) = trailing(&data.status, self.now, data.active_at, &theme);
        let ink = if data.status.title_is_ink() || self.open { theme.foreground } else { theme.muted_foreground };
        let words_of_status = data.status.words();
        let mark_id = self.id.clone();
        // Hidden until the pointer is on the row, then it stands left of the time and the title gives it room; a press
        // does not open the session.
        let more_open = self.more_open;
        let hovered = window.use_keyed_state((self.id.clone(), "hovered"), cx, |_, _| false);
        let has_more = (self.on_more.is_some() || self.on_archive.is_some()) && (more_open || *hovered.read(cx));
        let archive = self.on_archive.map(|(archived, press)| {
            crate::button::Button::new((self.id.clone(), "archive-button"))
                .debug_name("session-archive")
                .icon(if archived { IconName::Unarchive } else { IconName::Archive })
                .variant(crate::button::ButtonVariant::Ghost)
                .size(crate::button::ButtonSize::IconSm)
                .tooltip(if archived { "Unarchive" } else { "Archive" })
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    press(window, cx)
                })
        });
        let more = self.on_more.map(|press| {
            div()
                .relative()
                .child(
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
                .children(self.more_menu)
        });
        let ends = has_more.then(|| {
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap(px(2.))
                .children(archive)
                .children(more)
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
            .on_hover(move |on, _, cx| {
                hovered.update(cx, |h, cx| {
                    if *h != *on {
                        *h = *on;
                        cx.notify();
                    }
                })
            })
            .when_some(self.on_open, |d, open| {
                d.on_click(move |_, window, cx| open(window, cx))
            })
            .when(self.open, |d| {
                d.child(
                    div()
                        .absolute()
                        .left(px(12.))
                        .top(px((ROW_HEIGHT - 16.) / 2.))
                        .w(px(3.))
                        .h(px(16.))
                        .rounded_full()
                        .bg(theme.foreground)
                        .debug_selector(|| "session-row-open".into()),
                )
            })
            .child(
                div()
                    .id((mark_id.clone(), "status"))
                    .tooltip(Tooltip::text(words_of_status))
                    .child(mark),
            )
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
            .children(ends)
            // Short words line up in a column.
            .when(
                self.show_time
                    || !matches!(
                        data.status,
                        SessionStatus::Idle | SessionStatus::Working | SessionStatus::Finished
                    ),
                |d| {
                    d.child(
                        div()
                            .debug_selector(|| "row-time".into())
                            .flex_none()
                            .min_w(px(26.))
                            .max_w(px(150.))
                            .truncate()
                            .text_right()
                            .text_size(TextSize::Xs.font_size())
                            .text_color(tone)
                            .child(words),
                    )
                },
            )
    }
}
