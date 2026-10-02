use gpui_kit::{AnyElement, Context, IntoElement, ParentElement, Styled, Window, div};

use crate::scale::px;
use super::super::{Sidebar, SidebarEvent};
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::IconName,
    keys,
    menu::{self, Choice, Entry, Menu, MenuItem, MenuLook, Origin},
    popover::{Hang, Popover},
    segmented::{Segment, Segmented},
    sidebar_filter::{SessionFilter, describe, hidden_by},
    sidebar_layout::SidebarLayout,
    sidebar_model::ListMode,
};
use super::structs::HeadButton;

impl Sidebar {
    /// Lists the sessions by project, or in one list by priority. Emits `SidebarEvent::OptionsChanged`.
    pub fn choose_mode(&mut self, mode: ListMode, cx: &mut Context<Self>) {
        self.set_layout(SidebarLayout { mode, ..self.layout }, cx);
        cx.emit(SidebarEvent::LayoutChanged(self.layout));
    }

    /// Chooses which sessions the sidebar lists. Emits `SidebarEvent::OptionsChanged`.
    pub fn choose_filter(&mut self, filter: SessionFilter, cx: &mut Context<Self>) {
        self.filter_menu = false;
        self.set_layout(SidebarLayout { filter, ..self.layout }, cx);
        cx.emit(SidebarEvent::LayoutChanged(self.layout));
    }

    /// One menu at a time: opening one shuts the other.
    fn open_head_menu(&mut self, filter: bool, cx: &mut Context<Self>) {
        self.filter_menu = filter && !self.filter_menu;
        self.add_menu = !filter && !self.add_menu;
        cx.notify();
    }

    fn head_button(look: HeadButton, press: impl Fn(&mut Window, &mut gpui_kit::App) + 'static) -> AnyElement {
        let HeadButton { id, icon, tip, lit, open, menu } = look;
        div()
            .relative()
            .child(
                Button::new(id)
                    .debug_name(id)
                    .icon(icon)
                    .variant(if lit { ButtonVariant::Secondary } else { ButtonVariant::Ghost })
                    .size(ButtonSize::IconSm)
                    .tooltip(tip)
                    .open(open)
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        press(window, cx)
                    }),
            )
            .children(menu)
            .into_any_element()
    }

    /// "Projects | Priority".
    fn mode_switch(&self, cx: &mut Context<Self>) -> AnyElement {
        let modes = [ListMode::Projects, ListMode::Priority];
        let this = cx.entity().downgrade();
        Segmented::new(
            "list-mode",
            modes.iter().map(|m| {
                Segment::new(m.words()).debug_name(match m {
                    ListMode::Projects => "list-mode-projects",
                    ListMode::Priority => "list-mode-priority",
                })
            }),
            modes.iter().position(|m| *m == self.layout.mode).unwrap_or(0),
        )
        .on_change(move |i, _, cx| drop(this.update(cx, |s, cx| s.choose_mode(modes[i], cx))))
        .into_any_element()
    }

    fn filter_menu_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let this = cx.entity().downgrade();
        let hidden = hidden_by(&self.all, SessionFilter::Active);
        let entries: Vec<Entry> = SessionFilter::ALL
            .into_iter()
            .map(|choice| {
                let pick = this.clone();
                let words = if choice == SessionFilter::Archived && hidden > 0 { format!("{} ({hidden})", choice.words()) } else { choice.words().to_string() };
                Entry::from(
                    MenuItem::new(words)
                        .debug_name(choice.row())
                        .choice(Choice::Radio(choice == self.layout.filter))
                        .on_select(move |_, cx| drop(pick.update(cx, |s, cx| s.choose_filter(choice, cx)))),
                )
            })
            .collect();
        let close = this.clone();
        Popover::new("filter-menu-popover")
            .open(true)
            .hang(Hang::Right(0., 30.))
            .keep_focus()
            .height(menu::height_in(MenuLook::PROJECT, SessionFilter::ALL.len()))
            .on_close(move |_, cx| drop(close.update(cx, |s, cx| {
                s.filter_menu = false;
                cx.notify();
            })))
            .child(Menu::new("filter-menu-panel", entries).look(MenuLook::PROJECT).origin(Origin::TopRight))
            .into_any_element()
    }

    fn add_menu_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let this = cx.entity().downgrade();
        let ask = |label: &'static str, debug: &'static str, cap: &'static str, event: SidebarEvent| {
            let sidebar = this.clone();
            Entry::from(MenuItem::new(label).debug_name(debug).cap(keys::cap(cap)).on_select(move |_, cx| {
                drop(sidebar.update(cx, |s, cx| {
                    s.add_menu = false;
                    cx.emit(event.clone());
                    cx.notify();
                }))
            }))
        };
        let entries = vec![
            ask("Open folder…", "add-folder", "⌘o", SidebarEvent::AddFolder),
            ask("Open over SSH…", "add-ssh", "⌘⇧o", SidebarEvent::AddRemote),
        ];
        let close = this.clone();
        Popover::new("add-menu-popover")
            .open(true)
            .hang(Hang::Right(0., 30.))
            .keep_focus()
            .height(menu::height_in(MenuLook::PROJECT, 2))
            .on_close(move |_, cx| drop(close.update(cx, |s, cx| {
                s.add_menu = false;
                cx.notify();
            })))
            .child(Menu::new("add-menu-panel", entries).look(MenuLook::PROJECT).origin(Origin::TopRight))
            .into_any_element()
    }

    /// The head: the mode switch at the left, the filter and add buttons at the right.
    pub(in super::super) fn head(&self, cx: &mut Context<Self>) -> AnyElement {
        let filter = self.layout.filter;
        let (toggle_filter, toggle_add) = (cx.entity().downgrade(), cx.entity().downgrade());
        let buttons = div()
            .flex()
            .items_center()
            .gap(px(4.))
            .child(Self::head_button(
                HeadButton {
                    id: "filter-button",
                    icon: IconName::FilterList,
                    tip: describe(filter),
                    lit: !filter.is_default(),
                    open: self.filter_menu,
                    menu: self.filter_menu.then(|| self.filter_menu_panel(cx)),
                },
                move |_, cx| drop(toggle_filter.update(cx, |s, cx| s.open_head_menu(true, cx))),
            ))
            .child(Self::head_button(
                HeadButton { id: "add-project", icon: IconName::Add, tip: "Add a project".into(), lit: false, open: self.add_menu, menu: self.add_menu.then(|| self.add_menu_panel(cx)) },
                move |_, cx| drop(toggle_add.update(cx, |s, cx| s.open_head_menu(false, cx))),
            ));
        div()
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .h(px(36.))
            .pl(px(8.))
            .pr(px(2.))
            .child(self.mode_switch(cx))
            .child(buttons)
            .into_any_element()
    }
}
