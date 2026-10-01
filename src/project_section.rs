//! The header of a project in the sidebar: a fold arrow, a mark for where it lives (a folder for a
//! project on this machine, a host mark and the host's name for one over SSH), its name, its branch, how
//! a remote project is connected, a `+` to start a session in it and a `⋯` for its menu.
//!
//! A remote project that is connecting or reconnecting shows a spinner and the word; one that is offline
//! shows the danger mark, "Offline" and a Retry action. A connected one shows nothing extra.
use std::rc::Rc;

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::{Icon, IconName},
    menu::{self, Entry, Menu, MenuItem, Origin, Tone},
    sidebar_model::{Connection, Location, ProjectData},
    spinner::Spinner,
    popover::{Hang, Popover},
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

pub use crate::session_row::ROW_HEIGHT;

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;
type Chooser = Rc<dyn Fn(MenuChoice, &mut Window, &mut App)>;

/// The words for how a project is connected: `None` when nothing needs saying.
pub fn connection_words(connection: Connection) -> Option<&'static str> {
    match connection {
        Connection::Connected => None,
        Connection::Connecting => Some("Connecting…"),
        Connection::Reconnecting => Some("Reconnecting…"),
        Connection::Offline => Some("Offline"),
    }
}

/// Where the menu unfolds from: the panel hangs from the `⋯` button at its right, so it grows out of its top-right
/// corner, under the button the reader pressed, and not from the far left.
pub const MENU_ORIGIN: Origin = Origin::TopRight;

/// What the `⋯` menu offers, in order.
pub const MENU: [&str; 6] = ["Pull requests", "Tasks", "Choose an icon…", "Close project", "Files", "Copy path"];

/// Which of the menu's entries a press chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuChoice {
    PullRequests,
    Tasks,
    ChooseIcon,
    Close,
    Files,
    CopyPath,
}

/// Why `choice` cannot be chosen now, or `None` when it can: pull requests need a forge remote.
pub fn unavailable(choice: MenuChoice, pulls: Option<&str>) -> Option<&str> {
    (choice == MenuChoice::PullRequests).then_some(pulls).flatten()
}

impl MenuChoice {
    pub const ALL: [MenuChoice; 6] = [Self::PullRequests, Self::Tasks, Self::ChooseIcon, Self::Close, Self::Files, Self::CopyPath];

    /// The key that reaches the same thing from anywhere, shown on the menu row.
    pub fn cap(self) -> Option<&'static str> {
        match self {
            Self::PullRequests => Some("⌘⇧p"),
            Self::Tasks => Some("⌘⇧l"),
            Self::ChooseIcon | Self::Close | Self::Files | Self::CopyPath => None,
        }
    }

    pub fn words(self) -> &'static str {
        MENU[Self::ALL.iter().position(|c| *c == self).unwrap_or(0)]
    }
}

#[derive(IntoElement)]
pub struct ProjectSection {
    id: ElementId,
    project: ProjectData,
    expanded: bool,
    selected: bool,
    menu_open: bool,
    on_toggle: Option<Handler>,
    on_new: Option<Handler>,
    on_retry: Option<Handler>,
    on_menu: Option<Handler>,
    on_menu_close: Option<Handler>,
    on_choose: Option<Chooser>,
}

impl ProjectSection {
    pub fn new(id: impl Into<ElementId>, project: ProjectData) -> Self {
        Self {
            id: id.into(),
            project,
            expanded: true,
            selected: false,
            menu_open: false,
            on_toggle: None,
            on_new: None,
            on_retry: None,
            on_menu: None,
            on_menu_close: None,
            on_choose: None,
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn menu_open(mut self, open: bool) -> Self {
        self.menu_open = open;
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }

    pub fn on_new_session(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_new = Some(Rc::new(f));
        self
    }

    pub fn on_retry(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_retry = Some(Rc::new(f));
        self
    }

    /// The `⋯` or a right-click asks for the menu, or closes it when it is open.
    pub fn on_menu(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_menu = Some(Rc::new(f));
        self
    }

    /// Closes the menu. The popover calls it on Escape, on a press outside, and when the row stops drawing it (the reader
    /// chose a row): it must shut the menu and never toggle it, or the last of these opens it again.
    pub fn on_menu_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_menu_close = Some(Rc::new(f));
        self
    }

    pub fn on_choose(mut self, f: impl Fn(MenuChoice, &mut Window, &mut App) + 'static) -> Self {
        self.on_choose = Some(Rc::new(f));
        self
    }
}

fn chip(text: SharedString, theme: &crate::theme::Theme) -> impl IntoElement {
    div()
        .flex_none()
        .px(px(6.))
        .h(px(18.))
        .flex()
        .items_center()
        .rounded_full()
        .bg(theme.card_strong)
        .font_family(MONO_FONT_FAMILY)
        .text_size(px(10.))
        .text_color(theme.muted_foreground)
        .child(text)
}

impl RenderOnce for ProjectSection {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let project = self.project;
        let host = match &project.location {
            Location::Local => None,
            Location::Ssh { host } => Some(host.clone()),
        };
        let words = connection_words(project.connection);
        let id = self.id.clone();
        let menu = self.menu_open.then(|| {
            let choose = self.on_choose.clone();
            let pulls = project.pulls_unavailable.clone();
            let mut extra = 0.;
            let entries: Vec<Entry> = MenuChoice::ALL
                .into_iter()
                .map(|choice| {
                    let choose = choose.clone();
                    let off = unavailable(choice, pulls.as_deref());
                    let item = MenuItem::new(choice.words())
                        .debug_name(format!("project-menu-{}", choice.words()))
                        .disabled(off.is_some())
                        .tone(if choice == MenuChoice::Close { Tone::Destructive } else { Tone::Default })
                        .on_select(move |window, cx| {
                            if let Some(choose) = &choose {
                                choose(choice, window, cx);
                            }
                        });
                    let item = match choice.cap() {
                        Some(cap) => item.cap(cap),
                        None => item,
                    };
                    Entry::from(match off {
                        Some(why) => {
                            extra += 18.;
                            item.description(SharedString::from(why.to_string()))
                        }
                        None => item,
                    })
                })
                .collect();
            let rows = entries.len();
            let panel = Menu::new((id.clone(), "menu-panel"), entries).look(menu::MenuLook::PROJECT).origin(MENU_ORIGIN);
            let close = self.on_menu_close.clone().or_else(|| self.on_menu.clone());
            Popover::new((id.clone(), "menu"))
                .open(true)
                .hang(Hang::Right(8., ROW_HEIGHT - 2.))
                .keep_focus()
                .height(menu::height_in(menu::MenuLook::PROJECT, rows) + extra)
                .on_close(move |window, cx| {
                    if let Some(toggle) = &close {
                        toggle(window, cx);
                    }
                })
                .child(panel)
        });
        let menu_handler = self.on_menu.clone();
        div()
            .id(self.id.clone())
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(6.))
            .h(px(ROW_HEIGHT))
            .px(px(10.))
            .w_full()
            .rounded(radius::md())
            .cursor_pointer()
            .text_size(TextSize::Sm.font_size())
            .when(self.selected, |d| d.bg(theme.card_strong))
            .hover(|s| s.bg(theme.card_strong.opacity(0.6)))
            .when_some(self.on_toggle, |d, toggle| d.on_click(move |_, window, cx| toggle(window, cx)))
            .when_some(menu_handler, |d, open| d.on_mouse_down(MouseButton::Right, move |_, window, cx| open(window, cx)))
            .child(
                Icon::new(if self.expanded { IconName::ChevronDown } else { IconName::ChevronRight })
                    .size(px(16.))
                    .color(muted),
            )
            .child(crate::project_badge::ProjectBadge::new(project.badge.label.clone(), project.badge.color).icon(project.badge.icon.clone()))
            .child(div().flex_none().max_w(px(160.)).truncate().font_weight(FontWeight::MEDIUM).child(project.name))
            .when_some(host, |d, host| d.child(chip(host, &theme)))
            .when_some(project.branch, |d, branch| {
                d.child(div().min_w_0().truncate().font_family(MONO_FONT_FAMILY).text_size(px(11.)).text_color(muted).child(branch))
            })
            .child(div().flex_1())
            .when_some(words, |d, words| {
                let offline = project.connection == Connection::Offline;
                d.child(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .gap(px(4.))
                        .text_size(TextSize::Xs.font_size())
                        .text_color(if offline { theme.danger } else { muted })
                        .child(if offline {
                            Icon::new(IconName::Error).size(px(12.)).color(theme.danger).into_any_element()
                        } else {
                            Spinner::new((id.clone(), "connecting")).size(px(12.)).into_any_element()
                        })
                        .child(words),
                )
            })
            .when(project.connection == Connection::Offline, |d| {
                d.when_some(self.on_retry, |d, retry| {
                    d.child(
                        Button::new((id.clone(), "retry"))
                            .label("Retry")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .on_click(move |_, window, cx| {
                                cx.stop_propagation();
                                retry(window, cx)
                            }),
                    )
                })
            })
            .when_some(self.on_new, |d, new| {
                d.child(
                    Button::new((id.clone(), "new"))
                        .icon(IconName::Add)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("New session")
                        .on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            new(window, cx)
                        }),
                )
            })
            .when_some(self.on_menu, |d, open| {
                d.child(
                    Button::new((id.clone(), "menu"))
                        .icon(IconName::MoreHoriz)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("More")
                        .open(self.menu_open)
                        .on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            open(window, cx)
                        }),
                )
            })
            .children(menu)
    }
}

#[cfg(test)]
mod tests;
