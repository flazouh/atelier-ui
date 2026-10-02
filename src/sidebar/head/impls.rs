use gpui_kit::{AnyElement, Context, IntoElement, ParentElement, Styled, Window, div};

use crate::scale::px;
use super::super::{Sidebar, SidebarEvent};
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::IconName,
    keys,
    menu::{self, Choice, Entry, Menu, MenuItem, MenuLook, Origin},
    popover::{Hang, Popover},
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
        self.options_menu = false;
        self.set_layout(
            SidebarLayout {
                filter,
                ..self.layout
            },
            cx,
        );
        cx.emit(SidebarEvent::LayoutChanged(self.layout));
    }

    /// One menu at a time: opening one shuts the other.
    fn open_head_menu(&mut self, options: bool, cx: &mut Context<Self>) {
        self.options_menu = options && !self.options_menu;
        self.add_menu = !options && !self.add_menu;
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

    fn options_menu_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let this = cx.entity().downgrade();
        let hidden = hidden_by(&self.all, SessionFilter::Active);
        let modes = [ListMode::Projects, ListMode::Priority].into_iter().map(|mode| {
            let pick = this.clone();
            Entry::from(
                MenuItem::new(mode.words())
                    .debug_name(match mode {
                        ListMode::Projects => "list-mode-projects",
                        ListMode::Priority => "list-mode-priority",
                    })
                    .choice(Choice::Radio(mode == self.layout.mode))
                    .on_select(move |_, cx| {
                        drop(pick.update(cx, |s, cx| {
                            s.options_menu = false;
                            s.choose_mode(mode, cx)
                        }))
                    }),
            )
        });
        let filters = SessionFilter::ALL.into_iter().map(|choice| {
            let pick = this.clone();
            let words = if choice == SessionFilter::Archived && hidden > 0 {
                format!("{} ({hidden})", choice.words())
            } else {
                choice.words().to_string()
            };
            Entry::from(
                MenuItem::new(words)
                    .debug_name(choice.row())
                    .choice(Choice::Radio(choice == self.layout.filter))
                    .on_select(move |_, cx| {
                        drop(pick.update(cx, |s, cx| s.choose_filter(choice, cx)))
                    }),
            )
        });
        let entries: Vec<Entry> = std::iter::once(Entry::Label("List by".into()))
            .chain(modes)
            .chain([Entry::Separator, Entry::Label("Show".into())])
            .chain(filters)
            .collect();
        let close = this.clone();
        Popover::new("options-menu-popover")
            .open(true)
            .hang(Hang::Right(0., 30.))
            .keep_focus()
            .height(menu::height_of(MenuLook::PROJECT, &entries))
            .on_close(move |_, cx| {
                drop(close.update(cx, |s, cx| {
                    s.options_menu = false;
                    cx.notify();
                }))
            })
            .child(
                Menu::new("options-menu-panel", entries)
                    .look(MenuLook::PROJECT)
                    .origin(Origin::TopRight),
            )
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

    /// The head: at the right, the ⋯ with how the sidebar lists (lit while a filter hides sessions) and the add button.
    pub(in super::super) fn head(&self, cx: &mut Context<Self>) -> AnyElement {
        let filter = self.layout.filter;
        let (toggle_options, toggle_add) = (cx.entity().downgrade(), cx.entity().downgrade());
        let buttons = div()
            .flex()
            .items_center()
            .gap(px(4.))
            .child(Self::head_button(
                HeadButton {
                    id: "sidebar-options",
                    icon: IconName::MoreHoriz,
                    tip: if filter.is_default() { "Sidebar options".into() } else { describe(filter) },
                    lit: !filter.is_default(),
                    open: self.options_menu,
                    menu: self.options_menu.then(|| self.options_menu_panel(cx)),
                },
                move |_, cx| drop(toggle_options.update(cx, |s, cx| s.open_head_menu(true, cx))),
            ))
            .child(Self::head_button(
                HeadButton { id: "add-project", icon: IconName::Add, tip: "Add a project".into(), lit: false, open: self.add_menu, menu: self.add_menu.then(|| self.add_menu_panel(cx)) },
                move |_, cx| drop(toggle_add.update(cx, |s, cx| s.open_head_menu(false, cx))),
            ));
        div()
            .flex_none()
            .flex()
            .items_center()
            .justify_end()
            .h(px(36.))
            .pr(px(2.))
            .child(buttons)
            .into_any_element()
    }
}
