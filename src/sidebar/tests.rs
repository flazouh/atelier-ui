use gpui_kit::{Entity, Modifiers, TestAppContext, VisualTestContext, point, px, size};

use super::*;
use crate::{
    agent_look::AgentLook,
    session_status::SessionStatus,
    sidebar_model::{Connection, Location, SessionData},
    theme::{Appearance, set_appearance},
};

fn projects() -> Vec<ProjectData> {
    let look = AgentLook::neutral(&crate::theme::Theme::light());
    vec![ProjectData {
        id: "atelier".into(),
        name: "atelier".into(),
        location: Location::Local,
        connection: Connection::Connected,
        sessions: vec![SessionData {
            archived: false,
            in_panel: false,
            provider: None,
            id: "s".into(),
            title: "A session".into(),
            look,
            status: SessionStatus::Idle,
            active_at: 1,
        }],
        pulls_unavailable: None,
        badge: Default::default(),
    }]
}

fn open(cx: &mut TestAppContext) -> (Entity<Sidebar>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let (sidebar, cx) = cx.add_window_view(|_, cx| Sidebar::new(cx));
    cx.simulate_resize(size(px(400.), px(600.)));
    sidebar.update(cx, |s, cx| s.set_projects(projects(), 10, cx));
    frames(&sidebar, cx, 4);
    (sidebar, cx)
}

fn frames(sidebar: &Entity<Sidebar>, cx: &mut VisualTestContext, n: usize) {
    for _ in 0..n {
        cx.run_until_parked();
        sidebar.update(cx, |_, cx| cx.notify());
    }
}

/// A project's menu closed by a choice stayed on screen: the popover, no longer drawn, asked to close, and its close toggled the
/// menu open again.
#[gpui_kit::test]
fn a_choice_in_the_project_menu_closes_it_and_it_stays_closed(cx: &mut TestAppContext) {
    let (sidebar, cx) = open(cx);
    sidebar.update(cx, |s, cx| {
        s.menu = Some("atelier".into());
        cx.notify();
    });
    frames(&sidebar, cx, 4);
    let row = cx.debug_bounds("project-menu-Copy path").expect("the menu is open").center();
    cx.simulate_click(row, Modifiers::default());
    frames(&sidebar, cx, 12);
    assert!(sidebar.read_with(cx, |s, _| s.menu.is_none()), "the choice shut the menu");
    assert!(cx.debug_bounds("project-menu-Copy path").is_none(), "and it is not drawn");
    let _ = point(px(0.), px(0.));
}

#[gpui_kit::test]
fn escape_closes_the_project_menu(cx: &mut TestAppContext) {
    let (sidebar, cx) = open(cx);
    sidebar.update(cx, |s, cx| {
        s.menu = Some("atelier".into());
        cx.notify();
    });
    frames(&sidebar, cx, 4);
    cx.simulate_keystrokes("escape");
    frames(&sidebar, cx, 12);
    assert!(sidebar.read_with(cx, |s, _| s.menu.is_none()));
    assert!(cx.debug_bounds("project-menu-Copy path").is_none());
}

fn look() -> AgentLook {
    let strip = crate::sprite::Strip { path: "icons/bot.svg", bytes: b"", frames: 1, frame_ms: 1000, loops: false };
    let color = gpui_kit::Hsla::default();
    AgentLook {
        mark: crate::agent_look::Mark { working: strip, orbiting: strip, color, icon_frame: 0 },
        message: color,
        glimmer: color,
        labels: crate::agent_look::PhaseLabels::default(),
    }
}

fn project(sessions: &[(&str, u64)]) -> Vec<ProjectData> {
    let sessions = sessions
        .iter()
        .map(|(id, at)| SessionData { archived: false, in_panel: false, provider: None, id: (*id).into(), title: (*id).into(), look: look(), status: SessionStatus::Idle, active_at: *at })
        .collect();
    vec![ProjectData {
        id: "p".into(),
        name: "p".into(),
        location: Location::Local,
        connection: Connection::Connected,
        sessions,
        pulls_unavailable: None,
        badge: Default::default(),
    }]
}

fn order(sidebar: &Sidebar) -> Vec<String> {
    sidebar
        .rows()
        .iter()
        .filter_map(|row| match row {
            crate::sidebar_model::Row::Session { project, session } => Some(sidebar.projects[*project].sessions[*session].id.to_string()),
            _ => None,
        })
        .collect()
}

/// While the pointer is on the list, new activity does not move its rows; when the pointer leaves, the
/// list sorts again.
#[gpui_kit::test]
fn the_list_holds_its_order_under_the_pointer(cx: &mut TestAppContext) {
    use gpui_kit::AppContext;
    let sidebar = cx.new(Sidebar::new);
    sidebar.update(cx, |s, cx| s.set_projects(project(&[("a", 30), ("b", 20), ("c", 10)]), 100, cx));
    assert_eq!(sidebar.read_with(cx, |s, _| order(s)), ["a", "b", "c"]);
    sidebar.update(cx, |s, cx| s.hold(true, cx));
    sidebar.update(cx, |s, cx| s.set_projects(project(&[("a", 30), ("b", 20), ("c", 90)]), 100, cx));
    assert_eq!(sidebar.read_with(cx, |s, _| order(s)), ["a", "b", "c"], "c turned busy and stays put");
    sidebar.update(cx, |s, cx| s.hold(false, cx));
    assert_eq!(sidebar.read_with(cx, |s, _| order(s)), ["c", "a", "b"], "sorted again once the pointer leaves");
}

/// The head is the one place the options live: the switch and the filter change what the sidebar lists and say so in
/// one event; setting them from outside says nothing; and "all" stays whole under any filter.
#[gpui_kit::test]
fn the_head_holds_the_options_and_reports_a_change_once(cx: &mut TestAppContext) {
    use crate::{sidebar_filter::SessionFilter, sidebar_layout::SidebarLayout, sidebar_model::ListMode};
    let (sidebar, cx) = open(cx);
    let heard = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let log = heard.clone();
    let _sub = cx.update(|_, cx| {
        cx.subscribe(&sidebar, move |_, event: &SidebarEvent, _| {
            if let SidebarEvent::LayoutChanged(options) = event {
                log.borrow_mut().push(*options);
            }
        })
    });
    let mut data = project(&[("a", 30), ("b", 20)]);
    data[0].sessions[1].archived = true;
    sidebar.update(cx, |s, cx| s.set_projects(data, 100, cx));
    assert_eq!(sidebar.read_with(cx, |s, _| order(s)), ["a"], "an archived session is off the default list");
    assert_eq!(sidebar.read_with(cx, |s, _| s.all_projects()[0].sessions.len()), 2, "but the sidebar still holds it");
    sidebar.update(cx, |s, cx| s.choose_filter(SessionFilter::Archived, cx));
    assert_eq!(sidebar.read_with(cx, |s, _| order(s)), ["b"]);
    sidebar.update(cx, |s, cx| s.choose_mode(ListMode::Priority, cx));
    assert_eq!(*heard.borrow(), [SidebarLayout { mode: ListMode::Projects, filter: SessionFilter::Archived, ..Default::default() }, SidebarLayout { mode: ListMode::Priority, filter: SessionFilter::Archived, ..Default::default() }]);
    sidebar.update(cx, |s, cx| s.set_layout(SidebarLayout::default(), cx));
    assert_eq!(heard.borrow().len(), 2, "restoring the options from outside says nothing");
    assert_eq!(sidebar.read_with(cx, |s, _| order(s)), ["a"]);
}

/// One layout decides the rows: with the project's badge on, the time off and a fold of three, the sidebar draws and folds as
/// the layout says.
#[gpui_kit::test]
fn the_layout_decides_what_a_row_shows_and_how_much_folds(cx: &mut TestAppContext) {
    use crate::sidebar_layout::{BadgeShow, SidebarLayout};
    let (sidebar, cx) = open(cx);
    sidebar.update(cx, |s, cx| s.set_projects(project(&[("a", 60), ("b", 50), ("c", 40), ("d", 30), ("e", 20)]), 100, cx));
    frames(&sidebar, cx, 4);
    assert!(cx.debug_bounds("row-project").is_none(), "by project, Auto puts no badge on a row");
    assert_eq!(sidebar.read_with(cx, |s, _| order(s)).len(), 5);
    sidebar.update(cx, |s, cx| s.set_layout(SidebarLayout { project_badge: BadgeShow::Always, fold_after: 3, ..Default::default() }, cx));
    frames(&sidebar, cx, 4);
    assert!(cx.debug_bounds("row-project").is_some(), "Always puts it on every row, in either list");
    assert_eq!(sidebar.read_with(cx, |s, _| order(s)).len(), 3, "three sessions show and two fold");
}

/// The head holds no switch: its ⋯ opens the sidebar's options, how it lists and what it shows, and each choice
/// reports once.
#[gpui_kit::test]
fn the_head_options_live_behind_its_three_dots(cx: &mut TestAppContext) {
    use crate::{sidebar_filter::SessionFilter, sidebar_model::ListMode};
    let (sidebar, cx) = open(cx);
    let heard = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let log = heard.clone();
    let _sub = cx.update(|_, cx| {
        cx.subscribe(&sidebar, move |_, event: &SidebarEvent, _| {
            if let SidebarEvent::LayoutChanged(options) = event {
                log.borrow_mut().push(*options);
            }
        })
    });
    sidebar.update(cx, |s, cx| s.set_projects(project(&[("a", 30), ("b", 20)]), 100, cx));
    frames(&sidebar, cx, 4);
    assert!(cx.debug_bounds("list-mode-projects").is_none(), "no switch in the head");
    let press = |name: &'static str, cx: &mut gpui_kit::VisualTestContext| {
        let at = cx.debug_bounds(name).unwrap_or_else(|| panic!("{name} is drawn")).center();
        cx.simulate_click(at, Modifiers::default());
        frames(&sidebar, cx, 4);
    };
    press("sidebar-options", cx);
    press("list-mode-priority", cx);
    assert_eq!(sidebar.read_with(cx, |s, _| s.layout().mode), ListMode::Priority);
    assert!(cx.debug_bounds("list-mode-projects").is_none(), "a choice shuts the menu");
    press("sidebar-options", cx);
    press(SessionFilter::Archived.row(), cx);
    assert_eq!(sidebar.read_with(cx, |s, _| s.layout().filter), SessionFilter::Archived, "the last row is in reach");
    assert_eq!(heard.borrow().len(), 2, "one event for each choice");
}

fn targets() -> Vec<crate::menu::Branch> {
    use crate::menu::Branch;
    vec![Branch::with("claude", "Claude Code", vec![Branch::leaf("claude/me", "me@work"), Branch::leaf("claude/openrouter", "OpenRouter")]), Branch::leaf("codex", "Codex")]
}

fn open_handoff(cx: &mut TestAppContext) -> (Entity<Sidebar>, &mut VisualTestContext) {
    let (sidebar, cx) = open(cx);
    sidebar.update(cx, |s, cx| {
        s.set_handoff("atelier".into(), targets(), cx);
        s.session_menu = Some("s".into());
        cx.notify();
    });
    frames(&sidebar, cx, 4);
    (sidebar, cx)
}

#[gpui_kit::test]
fn handoff_in_a_sessions_menu_opens_the_agents_and_a_click_on_a_provider_hands_off_at_once(cx: &mut TestAppContext) {
    use std::{cell::RefCell, rc::Rc};
    let (sidebar, cx) = open_handoff(cx);
    let heard: Rc<RefCell<Vec<SidebarEvent>>> = Rc::default();
    let hearing = heard.clone();
    let _listening = cx.update(|_, cx| cx.subscribe(&sidebar, move |_, event: &SidebarEvent, _| hearing.borrow_mut().push(event.clone())));

    let handoff = cx.debug_bounds("session-menu-handoff").expect("the menu offers it").center();
    cx.simulate_mouse_move(handoff, None, Modifiers::default());
    frames(&sidebar, cx, 6);
    let agent = cx.debug_bounds("branch-claude").expect("the agents open beside it").center();
    cx.simulate_mouse_move(agent, None, Modifiers::default());
    frames(&sidebar, cx, 6);
    let provider = cx.debug_bounds("branch-claude/openrouter").expect("an agent with a choice opens its providers").center();
    cx.simulate_click(provider, Modifiers::default());
    frames(&sidebar, cx, 4);

    let target = SidebarEvent::Handoff { project: "atelier".into(), session: "s".into(), target: "claude/openrouter".into() };
    assert_eq!(*heard.borrow(), vec![target]);
    assert!(sidebar.read_with(cx, |s, _| s.session_menu.is_none()), "and the menu is shut");
}

#[gpui_kit::test]
fn handoff_with_nowhere_to_go_is_dimmed_and_hands_off_nothing(cx: &mut TestAppContext) {
    let (sidebar, cx) = open(cx);
    sidebar.update(cx, |s, cx| {
        s.session_menu = Some("s".into());
        cx.notify();
    });
    frames(&sidebar, cx, 4);

    let handoff = cx.debug_bounds("session-menu-handoff").expect("the row is there").center();
    cx.simulate_mouse_move(handoff, None, Modifiers::default());
    frames(&sidebar, cx, 6);
    assert!(cx.debug_bounds("branch-claude").is_none());
}

#[gpui_kit::test]
fn every_row_of_the_project_menu_has_an_icon(cx: &mut TestAppContext) {
    let (sidebar, cx) = open(cx);
    sidebar.update(cx, |s, cx| {
        s.menu = Some("atelier".into());
        cx.notify();
    });
    frames(&sidebar, cx, 4);
    for words in crate::project_section::MENU {
        let name: &'static str = Box::leak(format!("menu-icon-{words}").into_boxed_str());
        assert!(cx.debug_bounds(name).is_some(), "{words} has an icon");
    }
}

#[gpui_kit::test]
fn every_row_of_a_sessions_menu_has_an_icon(cx: &mut TestAppContext) {
    let (_, cx) = open_handoff(cx);
    for words in ["Archive", "Handoff", "Copy session id"] {
        let name: &'static str = Box::leak(format!("menu-icon-{words}").into_boxed_str());
        assert!(cx.debug_bounds(name).is_some(), "{words} has an icon");
    }
}

/// The app may draw who runs a session itself: the sidebar asks for each session's mark by its id, and puts what it gets
/// in the place of the agent's mark. It knows nothing of what the element shows.
#[gpui_kit::test]
fn a_session_the_app_gives_a_mark_for_shows_it_and_the_others_keep_the_agents_mark(cx: &mut TestAppContext) {
    use gpui_kit::{InteractiveElement, IntoElement, Styled, div};
    let (sidebar, cx) = open(cx);
    assert!(cx.debug_bounds("own-mark").is_none(), "no mark is given: the row is the agent's");
    sidebar.update(cx, |s, cx| {
        s.set_session_mark(|id, _, _| (id.as_ref() == "s").then(|| div().debug_selector(|| "own-mark".into()).size(px(20.)).into_any_element()), cx)
    });
    frames(&sidebar, cx, 4);
    assert!(cx.debug_bounds("own-mark").is_some(), "the session the app named shows its mark");
    sidebar.update(cx, |s, cx| s.set_session_mark(|_, _, _| None, cx));
    frames(&sidebar, cx, 4);
    assert!(cx.debug_bounds("own-mark").is_none(), "a session the app gives nothing for keeps the agent's mark");
}
