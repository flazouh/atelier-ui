use std::{collections::HashMap, ops::Range, time::Instant};

use gpui_kit::{
    AnyElement, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, Render, ScrollStrategy, SharedString,
    StatefulInteractiveElement, Styled, UniformListScrollHandle, Window, div,
    prelude::FluentBuilder, uniform_list,
};

use crate::scale::px;
use crate::{
    entrance::Entrance,
    focus::PressStop,
    icon::{Icon, IconName},
    keys::{self, Command, Press},
    motion::{Channel, Curve, Spring},
    project_section::{MenuChoice, ProjectSection},
    session_row::{ROW_HEIGHT, SessionRow},
    sidebar_filter::narrow,
    sidebar_layout::SidebarLayout,
    sidebar_model::{self, Activation, Folds, ListMode, Nav, ProjectData, Row, RowKey, key_of, position_of, rows_held},
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::{ENTERING, SidebarEvent};
use super::helpers::name;

pub struct Sidebar {
    /// Every project with every session, as the app gave them.
    pub(super) all: Vec<ProjectData>,
    /// What the filter leaves of them: the projects the rows are made from.
    pub(super) projects: Vec<ProjectData>,
    pub(super) folds: Folds,
    pub(super) rows: Vec<Row>,
    pub(super) selected: Option<RowKey>,
    /// The session the reader has open, by id.
    pub(super) open: Option<SharedString>,
    pub(super) now: u64,
    pub(super) scroll: UniformListScrollHandle,
    pub(super) focus: FocusHandle,
    pub(super) menu: Option<SharedString>,
    /// The session whose ⋯ menu is open, by id.
    pub(super) session_menu: Option<SharedString>,
    /// What the head chose, and whether the priority list shows all its earlier sessions.
    pub(super) layout: SidebarLayout,
    pub(super) earlier_open: bool,
    pub(super) filter_menu: bool,
    pub(super) add_menu: bool,
    pub(super) entering: HashMap<SharedString, Instant>,
    pub(super) moving: HashMap<RowKey, Channel>,
    /// While the pointer is on the list: each project's sessions in the order the rows had.
    pub(super) held: Option<HashMap<SharedString, Vec<SharedString>>>,
}

impl EventEmitter<SidebarEvent> for Sidebar {}

impl Focusable for Sidebar {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Sidebar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            all: Vec::new(),
            projects: Vec::new(),
            folds: Folds::default(),
            rows: Vec::new(),
            selected: None,
            open: None,
            now: 0,
            scroll: UniformListScrollHandle::new(),
            focus: cx.focus_handle(),
            menu: None,
            session_menu: None,
            layout: SidebarLayout::default(),
            earlier_open: false,
            filter_menu: false,
            add_menu: false,
            entering: HashMap::new(),
            moving: HashMap::new(),
            held: None,
        }
    }

    /// The pointer came onto the list, or left it. While it is on, the rows keep their order, so a row
    /// never moves under it; when it leaves, the list sorts again.
    pub fn hold(&mut self, on: bool, cx: &mut Context<Self>) {
        if on == self.held.is_some() {
            return;
        }
        if on {
            self.held = Some(self.shown_order());
        } else {
            self.held = None;
            self.refresh(cx);
        }
    }

    /// Each project's sessions in the order the rows show them.
    pub(super) fn shown_order(&self) -> HashMap<SharedString, Vec<SharedString>> {
        let mut order: HashMap<SharedString, Vec<SharedString>> = HashMap::new();
        for row in &self.rows {
            if let Row::Session { project, session } = *row {
                let p = &self.projects[project];
                order.entry(p.id.clone()).or_default().push(p.sessions[session].id.clone());
            }
        }
        order
    }

    /// The rows from the projects and the folds, in the held order while the pointer holds the list.
    pub(super) fn build_rows(&mut self) {
        self.rows = match self.layout.mode {
            ListMode::Projects => rows_held(&self.projects, &self.folds, self.held.as_ref(), self.layout.fold_after),
            ListMode::Priority => sidebar_model::priority_rows(&self.projects, self.earlier_open, self.layout.earlier_shown),
        };
        if self.held.is_some() {
            self.held = Some(self.shown_order());
        }
    }

    /// The projects as the app knows them now. A session that is new enters; one whose row moved slides.
    pub fn set_projects(&mut self, all: Vec<ProjectData>, now: u64, cx: &mut Context<Self>) {
        let projects = narrow(&all, self.layout.filter);
        let reduce = cx.reduce_motion();
        let before: HashMap<RowKey, usize> = self.rows.iter().enumerate().map(|(i, r)| (key_of(&self.projects, *r), i)).collect();
        let known: std::collections::HashSet<&SharedString> = self.projects.iter().flat_map(|p| p.sessions.iter().map(|s| &s.id)).collect();
        let fresh: Vec<SharedString> = if before.is_empty() {
            Vec::new()
        } else {
            projects.iter().flat_map(|p| p.sessions.iter()).filter(|s| !known.contains(&s.id)).map(|s| s.id.clone()).collect()
        };
        self.projects = projects;
        self.all = all;
        self.now = now;
        self.build_rows();
        let stamp = Instant::now();
        self.entering.retain(|_, at| stamp.duration_since(*at) < ENTERING);
        self.entering.extend(fresh.into_iter().map(|id| (id, stamp)));
        self.moving.retain(|_, channel| channel.is_running());
        for (at, row) in self.rows.iter().enumerate() {
            if !matches!(row, Row::Session { .. }) {
                continue;
            }
            let key = key_of(&self.projects, *row);
            if let Some(&was) = before.get(&key).filter(|&&was| was != at) {
                let mut slide = Channel::new((was as f32 - at as f32) * ROW_HEIGHT);
                slide.animate(0., Curve::Spring(Spring::LAYOUT), 0., reduce);
                self.moving.insert(key, slide);
            }
        }
        if let Some(key) = &self.selected
            && position_of(&self.projects, &self.rows, key).is_none()
        {
            self.selected = None;
        }
        cx.notify();
    }

    /// Sets the time "2m" is counted from. The app calls it about once a minute.
    /// Tells the sidebar which session the reader has open, so its row is marked.
    pub fn set_open(&mut self, id: Option<SharedString>, cx: &mut Context<Self>) {
        if self.open != id {
            self.open = id;
            cx.notify();
        }
    }

    /// The layout: the one value that says how the sidebar lists and draws.
    pub fn layout(&self) -> SidebarLayout {
        self.layout
    }

    /// Sets the layout, as when the app restores it or the Settings page changes it. No event: the app knows.
    pub fn set_layout(&mut self, layout: SidebarLayout, cx: &mut Context<Self>) {
        if self.layout != layout {
            let refilter = self.layout.filter != layout.filter;
            self.layout = layout;
            self.selected = None;
            if refilter {
                self.projects = narrow(&self.all, layout.filter);
            }
            self.refresh(cx);
        }
    }

    /// Every project with every session, before the filter: the island and the urgent key count from this.
    pub fn all_projects(&self) -> &[ProjectData] {
        &self.all
    }

    /// The session marked as the one in front, if any.
    pub fn open_session(&self) -> Option<&SharedString> {
        self.open.as_ref()
    }

    pub fn set_now(&mut self, now: u64, cx: &mut Context<Self>) {
        self.now = now;
        cx.notify();
    }

    /// Opens every project's fold of older sessions, so every session has a row. For measuring runs.
    pub fn open_all_older(&mut self, cx: &mut Context<Self>) {
        for project in &self.projects {
            if !self.folds.older_open(&project.id) {
                self.folds.toggle_older(&project.id);
            }
        }
        self.refresh(cx);
    }

    /// Scrolls the list so `top` pixels of rows are above the view. For measuring runs.
    pub fn set_scroll_top(&mut self, top: f32) {
        self.scroll.0.borrow().base_handle.set_offset(gpui_kit::point(px(0.), px(-top)));
    }

    pub fn projects(&self) -> &[ProjectData] {
        &self.projects
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn selected_row(&self) -> Option<usize> {
        self.selected.as_ref().and_then(|k| position_of(&self.projects, &self.rows, k))
    }

    /// Selects a session's row and scrolls to it. Does nothing when the session is not shown.
    pub fn select_session(&mut self, project: &str, session: &str, cx: &mut Context<Self>) {
        let key = RowKey::Session(project.to_string().into(), session.to_string().into());
        if let Some(at) = position_of(&self.projects, &self.rows, &key) {
            self.selected = Some(key);
            self.scroll.scroll_to_item(at, ScrollStrategy::Center);
            cx.notify();
        }
    }

    pub(super) fn refresh(&mut self, cx: &mut Context<Self>) {
        self.build_rows();
        cx.notify();
    }

    pub(super) fn select_row(&mut self, at: usize, cx: &mut Context<Self>) {
        if let Some(row) = self.rows.get(at) {
            self.selected = Some(key_of(&self.projects, *row));
            self.scroll.scroll_to_item(at, ScrollStrategy::Nearest);
            cx.notify();
        }
    }

    pub(super) fn toggle_project(&mut self, project: usize, cx: &mut Context<Self>) {
        let id = self.projects[project].id.clone();
        let collapsed = self.folds.is_collapsed(&id);
        self.folds.set_collapsed(&id, !collapsed);
        self.refresh(cx);
    }

    pub(super) fn navigate(&mut self, nav: Nav, cx: &mut Context<Self>) {
        let folds = self.folds.clone();
        let projects = self.projects.clone();
        let moved = sidebar_model::step(&self.rows, |p| folds.is_collapsed(&projects[p].id), self.selected_row(), nav);
        if let Some((project, collapse)) = moved.fold {
            let id = self.projects[project].id.clone();
            self.folds.set_collapsed(&id, collapse);
            self.refresh(cx);
        }
        if let Some(at) = moved.select {
            self.select_row(at, cx);
        }
    }

    pub(super) fn activate(&mut self, row: Row, cx: &mut Context<Self>) {
        match sidebar_model::activate(row) {
            Activation::OpenSession { project, session } => {
                cx.emit(SidebarEvent::Open {
                    project: self.projects[project].id.clone(),
                    session: self.projects[project].sessions[session].id.clone(),
                });
            }
            Activation::ToggleProject(project) => self.toggle_project(project, cx),
            Activation::ToggleOlder(project) => {
                let id = self.projects[project].id.clone();
                self.folds.toggle_older(&id);
                self.refresh(cx);
            }
            Activation::NewSession(project) => cx.emit(SidebarEvent::NewSession { project: self.projects[project].id.clone() }),
            Activation::OpenTasks(project) => cx.emit(SidebarEvent::Tasks { project: self.projects[project].id.clone() }),
            Activation::ToggleEarlier => {
                self.earlier_open = !self.earlier_open;
                self.refresh(cx);
            }
            Activation::Nothing => {}
        }
    }

    pub(super) fn choose(&mut self, project: usize, choice: MenuChoice, cx: &mut Context<Self>) {
        let project = self.projects[project].id.clone();
        self.menu = None;
        cx.emit(match choice {
            MenuChoice::PullRequests => SidebarEvent::PullRequests { project },
            MenuChoice::Tasks => SidebarEvent::Tasks { project },
            MenuChoice::ChooseIcon => SidebarEvent::ChooseIcon { project },
            MenuChoice::Close => SidebarEvent::CloseProject { project },
            MenuChoice::Files => SidebarEvent::OpenFiles { project },
            MenuChoice::CopyPath => SidebarEvent::CopyPath { project },
        });
        cx.notify();
    }

    pub(super) fn key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let nav = match event.keystroke.key.as_str() {
            "up" => Some(Nav::Up),
            "down" => Some(Nav::Down),
            "left" => Some(Nav::Left),
            "right" => Some(Nav::Right),
            "home" => Some(Nav::First),
            "end" => Some(Nav::Last),
            "escape" if self.menu.is_some() || self.session_menu.is_some() => {
                self.menu = None;
                self.session_menu = None;
                cx.notify();
                cx.stop_propagation();
                return;
            }
            "enter" | "space" => {
                if let Some(row) = self.selected_row().and_then(|at| self.rows.get(at).copied()) {
                    self.activate(row, cx);
                    cx.stop_propagation();
                }
                return;
            }
            _ => match keys::read_now(&Press::from_keystroke(&event.keystroke), cx) {
                Some(Command::NextFile) => Some(Nav::Down),
                Some(Command::PreviousFile) => Some(Nav::Up),
                _ => None,
            },
        };
        if let Some(nav) = nav {
            self.navigate(nav, cx);
            cx.stop_propagation();
        }
    }

    pub(super) fn row_element(&mut self, at: usize, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let this = cx.entity();
        let row = self.rows[at];
        let key = key_of(&self.projects, row);
        let selected = self.selected.as_ref() == Some(&key);
        match row {
            Row::Project { project } => {
                let data = self.projects[project].clone();
                let id = data.id.clone();
                let menu_open = self.menu.as_ref() == Some(&id);
                let (t, n, r, m, c) = (this.clone(), this.clone(), this.clone(), this.clone(), this.clone());
                ProjectSection::new(name("project", &id.clone()), data)
                    .expanded(!self.folds.is_collapsed(&id))
                    .selected(selected)
                    .menu_open(menu_open)
                    .on_toggle(move |_, cx| {
                        t.update(cx, |s, cx| {
                            s.select_row(at, cx);
                            s.toggle_project(project, cx)
                        })
                    })
                    .on_new_session(move |_, cx| n.update(cx, |s, cx| cx.emit(SidebarEvent::NewSession { project: s.projects[project].id.clone() })))
                    .on_retry(move |_, cx| r.update(cx, |s, cx| cx.emit(SidebarEvent::Retry { project: s.projects[project].id.clone() })))
                    .on_menu(move |_, cx| {
                        m.update(cx, |s, cx| {
                            let id = s.projects[project].id.clone();
                            s.menu = if s.menu.as_ref() == Some(&id) { None } else { Some(id) };
                            cx.notify();
                        })
                    })
                    .on_menu_close({
                        let shut = this.clone();
                        move |_, cx| {
                            shut.update(cx, |s, cx| {
                                s.menu = None;
                                cx.notify();
                            });
                        }
                    })
                    .on_choose(move |choice, _, cx| c.update(cx, |s, cx| s.choose(project, choice, cx)))
                    .into_any_element()
            }
            Row::Session { project, session } => {
                let data = self.projects[project].sessions[session].clone();
                let id = data.id.clone();
                let row_id = name("session", &id.clone());
                let open = this.clone();
                let sessions_entering = self.entering.get(&id).is_some_and(|at| at.elapsed() < ENTERING);
                let is_open = self.open.as_ref() == Some(&id);
                let archived = data.archived;
                let in_panel = data.in_panel;
                let menu_open = self.session_menu.as_ref() == Some(&id);
                let (project_id, session_id) = (self.projects[project].id.clone(), id.clone());
                let menu = menu_open.then(|| {
                    use crate::{
                        menu::{self, Entry, Menu, MenuItem, MenuLook, Origin},
                        popover::{Hang, Popover},
                    };
                    // A choice shuts the menu, then asks the app for its work.
                    let ask = |label: &'static str, debug: &'static str, event: SidebarEvent| {
                        let sidebar = this.clone();
                        Entry::from(MenuItem::new(label).debug_name(debug).on_select(move |_, cx| {
                            sidebar.update(cx, |s, cx| {
                                s.session_menu = None;
                                cx.emit(event.clone());
                                cx.notify();
                            })
                        }))
                    };
                    let target = |event: fn(SharedString, SharedString) -> SidebarEvent| event(project_id.clone(), session_id.clone());
                    let mut entries = vec![ask(
                        if archived { "Unarchive" } else { "Archive" },
                        "session-menu-archive",
                        SidebarEvent::Archive { project: project_id.clone(), session: session_id.clone(), archive: !archived },
                    )];
                    entries.push(ask("Copy session id", "session-menu-copy-id", target(|project, session| SidebarEvent::CopySessionId { project, session })));
                    if in_panel {
                        entries.push(ask("Close panel", "session-menu-close", target(|project, session| SidebarEvent::CloseSession { project, session })));
                    }
                    let rows = entries.len();
                    let close = this.clone();
                    Popover::new(name("session-menu-popover", &id))
                        .open(true)
                        .hang(Hang::Right(0., 24.))
                        .keep_focus()
                        .height(menu::height_in(MenuLook::PROJECT, rows))
                        .on_close(move |_, cx| {
                            close.update(cx, |s, cx| {
                                s.session_menu = None;
                                cx.notify();
                            })
                        })
                        .child(Menu::new(name("session-menu-panel", &id), entries).look(MenuLook::PROJECT).origin(Origin::TopRight))
                        .into_any_element()
                });
                let toggling = this.clone();
                let toggled = id.clone();
                let mut element = SessionRow::new(row_id.clone(), data, self.now)
                    .selected(selected)
                    .open(is_open)
                    .on_open(move |_, cx| open.update(cx, |s, cx| s.activate(Row::Session { project, session }, cx)))
                    .more(
                        menu_open,
                        move |_, cx| {
                            toggling.update(cx, |s, cx| {
                                s.session_menu = if s.session_menu.as_ref() == Some(&toggled) { None } else { Some(toggled.clone()) };
                                cx.notify();
                            })
                        },
                        menu,
                    );
                // The layout decides what the row shows and whether the project's badge is on it.
                element = element.layout(&self.layout);
                if self.layout.badge_on_rows() {
                    element = element.project(self.projects[project].badge.clone(), self.projects[project].name.clone());
                }
                let element = if sessions_entering {
                    Entrance::new(name("entering", &id), element).skip_initial(false).into_any_element()
                } else {
                    element.into_any_element()
                };
                let slide = self.moving.get(&key).map_or(0., Channel::value);
                if self.moving.get(&key).is_some_and(Channel::is_running) {
                    window.request_animation_frame();
                }
                div().relative().top(px(slide)).child(element).into_any_element()
            }
            Row::Section { section, count } => {
                let tone = match section {
                    sidebar_model::Section::NeedsYou => theme.warning,
                    sidebar_model::Section::Finished => theme.accent,
                    _ => theme.muted_foreground,
                };
                div()
                    .debug_selector(move || format!("section-{}", section.words()))
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(6.))
                    .h(px(ROW_HEIGHT))
                    .px(px(14.))
                    .text_size(TextSize::Xs.font_size())
                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                    .text_color(theme.muted_foreground)
                    .child(section.words())
                    .child(div().text_color(tone).child(count.to_string()))
                    .into_any_element()
            }
            Row::MoreEarlier { hidden, open } => {
                let t = this.clone();
                let words: SharedString = if open { "Show fewer".into() } else { format!("Show {hidden} more").into() };
                div()
                    .id("more-earlier")
                    .debug_selector(|| "more-earlier".into())
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(6.))
                    .h(px(ROW_HEIGHT))
                    .pl(px(14.))
                    .w_full()
                    .rounded(radius::md())
                    .cursor_pointer()
                    .text_size(TextSize::Xs.font_size())
                    .text_color(theme.muted_foreground)
                    .hover(|s| s.bg(theme.card_strong.opacity(0.6)))
                    .on_click(move |_, _, cx| t.update(cx, |s, cx| s.activate(row, cx)))
                    .child(Icon::new(if open { IconName::ChevronUp } else { IconName::ChevronDown }).size(px(14.)))
                    .child(words)
                    .into_any_element()
            }
            Row::Older { project, hidden, open } => {
                let t = this.clone();
                let words: SharedString = if open { "Show fewer".into() } else { format!("Show {hidden} older").into() };
                div()
                    .id(name("older", &self.projects[project].id.clone()))
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(6.))
                    .h(px(ROW_HEIGHT))
                    .pl(px(26.))
                    .w_full()
                    .rounded(radius::md())
                    .cursor_pointer()
                    .text_size(TextSize::Xs.font_size())
                    .text_color(theme.muted_foreground)
                    .when(selected, |d| d.bg(theme.card_strong))
                    .hover(|s| s.bg(theme.card_strong.opacity(0.6)))
                    .press_stop(name("older-focus", &self.projects[project].id.clone()), radius::md(), window, cx)
                    .on_click(move |_, _, cx| {
                        t.update(cx, |s, cx| {
                            s.select_row(at, cx);
                            s.activate(row, cx)
                        })
                    })
                    .child(Icon::new(if open { IconName::ChevronUp } else { IconName::ChevronDown }).size(px(14.)))
                    .child(words)
                    .into_any_element()
            }
            Row::Tasks { project } => {
                let t = this.clone();
                div()
                    .id(name("tasks", &self.projects[project].id.clone()))
                    .debug_selector(|| "sidebar-tasks".into())
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(8.))
                    .h(px(ROW_HEIGHT))
                    .pl(px(26.))
                    .w_full()
                    .rounded(radius::md())
                    .cursor_pointer()
                    .text_size(TextSize::Xs.font_size())
                    .text_color(theme.muted_foreground)
                    .when(selected, |d| d.bg(theme.card_strong))
                    .hover(|s| s.bg(theme.card_strong.opacity(0.6)))
                    .press_stop(name("tasks-focus", &self.projects[project].id.clone()), radius::md(), window, cx)
                    .on_click(move |_, _, cx| {
                        t.update(cx, |s, cx| {
                            s.select_row(at, cx);
                            s.activate(row, cx)
                        })
                    })
                    .child(crate::sidebar_model::tasks_words(self.projects[project].tasks_open))
                    .into_any_element()
            }
            Row::Empty { project } => {
                let t = this.clone();
                div()
                    .id(name("empty", &self.projects[project].id.clone()))
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(8.))
                    .h(px(ROW_HEIGHT))
                    .pl(px(26.))
                    .w_full()
                    .rounded(radius::md())
                    .cursor_pointer()
                    .text_size(TextSize::Xs.font_size())
                    .text_color(theme.muted_foreground)
                    .when(selected, |d| d.bg(theme.card_strong))
                    .hover(|s| s.bg(theme.card_strong.opacity(0.6)))
                    .press_stop(name("empty-focus", &self.projects[project].id.clone()), radius::md(), window, cx)
                    .on_click(move |_, _, cx| {
                        t.update(cx, |s, cx| {
                            s.select_row(at, cx);
                            s.activate(row, cx)
                        })
                    })
                    .child("No sessions yet")
                    .child(div().text_color(theme.foreground).child("New session"))
                    .into_any_element()
            }
        }
    }
}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.rows.len();
        let list = uniform_list("sidebar-rows", count, cx.processor(|this: &mut Self, range: Range<usize>, window, cx| {
            range.map(|at| this.row_element(at, window, cx)).collect::<Vec<_>>()
        }))
        .track_scroll(&self.scroll)
        .size_full();
        div()
            .id("sidebar")
            .key_context("Sidebar")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| this.key(event, window, cx)))
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| this.hold(*hovered, cx)))
            .on_mouse_down(
                gpui_kit::MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    if !this.focus.contains_focused(window, cx) {
                        window.focus(&this.focus, cx)
                    }
                }),
            )
            .size_full()
            .flex()
            .flex_col()
            .py(px(6.))
            .px(px(6.))
            .child(self.head(cx))
            .child(list)
    }
}
