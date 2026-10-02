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
        branch: Some("main".into()),
        sessions: vec![SessionData { archived: false, in_panel: false, id: "s".into(), title: "A session".into(), look, status: SessionStatus::Idle, active_at: 1 }],
        pulls_unavailable: None,
        tasks_open: None,
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
        .map(|(id, at)| SessionData { archived: false, in_panel: false, id: (*id).into(), title: (*id).into(), look: look(), status: SessionStatus::Idle, active_at: *at })
        .collect();
    vec![ProjectData { id: "p".into(), name: "p".into(), location: Location::Local, connection: Connection::Connected, branch: None, sessions, pulls_unavailable: None, tasks_open: None , badge: Default::default() }]
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

/// Each project ends with a tasks row; a press on it asks for that project's tasks.
#[gpui_kit::test]
fn a_press_on_the_tasks_row_asks_for_the_tasks_of_its_project(cx: &mut TestAppContext) {
    let (sidebar, cx) = open(cx);
    let told = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let heard = told.clone();
    let _sub = cx.update(|_, cx| {
        cx.subscribe(&sidebar, move |_, event: &SidebarEvent, _| {
            if let SidebarEvent::Tasks { project } = event {
                heard.borrow_mut().push(project.to_string());
            }
        })
    });
    let row = cx.debug_bounds("sidebar-tasks").expect("the tasks row is drawn").center();
    cx.simulate_click(row, Modifiers::default());
    frames(&sidebar, cx, 4);
    assert_eq!(told.borrow().len(), 1, "one press, one ask");
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
