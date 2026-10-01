use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui_kit::{Entity, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use super::*;
use crate::{
    agent_look::AgentLook,
    motion,
    sidebar_model::{Connection, Location, SessionData},
    theme::{Appearance, set_appearance},
};

fn session(id: &str, status: SessionStatus, at: u64) -> SessionData {
    SessionData { archived: false, in_panel: false, id: id.to_string().into(), title: id.to_string().into(), look: AgentLook::neutral(&crate::theme::Theme::light()), status, active_at: at }
}

fn project(id: &str, sessions: Vec<SessionData>) -> ProjectData {
    ProjectData {
        id: id.to_string().into(),
        name: id.to_string().into(),
        location: Location::Local,
        connection: Connection::Connected,
        branch: None,
        sessions,
        pulls_unavailable: None,
        tasks_open: None,
        badge: Default::default(),
    }
}

#[test]
fn the_shell_spring_is_apples_duration_08_bounce_02() {
    let omega = (SHELL.stiffness / SHELL.mass).sqrt();
    assert!((2. * std::f32::consts::PI / omega - 0.8).abs() < 1e-3, "a period of 0.8s");
    let zeta = SHELL.damping / (2. * (SHELL.stiffness * SHELL.mass).sqrt());
    assert!((zeta - 0.8).abs() < 1e-3, "bounce 0.2 is a damping ratio of 0.8");
}

#[test]
fn the_counts_are_the_sessions_in_each_state_across_every_project() {
    let projects = vec![
        project("a", vec![session("1", SessionStatus::Working, 5), session("2", SessionStatus::NeedsYou(Need::Approval), 9), session("3", SessionStatus::Idle, 1)]),
        project("b", vec![session("4", SessionStatus::Finished, 2), session("5", SessionStatus::Working, 3), session("6", SessionStatus::Failed("x".into()), 4)]),
    ];
    assert_eq!(counts_of(&projects), IslandCounts { running: 2, needs: 1, done: 1 });
    assert!(IslandCounts::default().is_empty());
}

#[test]
fn a_press_goes_to_the_session_that_needs_the_reader_most() {
    let projects = vec![
        project("a", vec![session("work", SessionStatus::Working, 50), session("done", SessionStatus::Finished, 40)]),
        project("b", vec![session("ask", SessionStatus::NeedsYou(Need::Question), 30), session("approve-old", SessionStatus::NeedsYou(Need::Approval), 10), session("approve-new", SessionStatus::NeedsYou(Need::Approval), 20)]),
    ];
    assert_eq!(most_urgent(&projects), Some(("b".into(), "approve-new".into())), "an approval first, the latest of them");
    let no_approval = vec![project("a", vec![session("work", SessionStatus::Working, 50), session("done", SessionStatus::Finished, 40)])];
    assert_eq!(most_urgent(&no_approval), Some(("a".into(), "done".into())), "finished before working");
    assert_eq!(most_urgent(&[project("a", vec![session("i", SessionStatus::Idle, 1)])]), None);
}

struct Page {
    counts: IslandCounts,
    log: Rc<RefCell<Vec<&'static str>>>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let log = self.log.clone();
        div().p(px(20.)).flex().items_start().child(SessionsIsland::new("island", self.counts).on_press(move |_, _| log.borrow_mut().push("press")))
    }
}

fn open(reduce: bool, counts: IslandCounts, cx: &mut TestAppContext) -> (Entity<Page>, &mut VisualTestContext, Rc<RefCell<Vec<&'static str>>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let l = log.clone();
    let (page, cx) = cx.add_window_view(move |_, _| Page { counts, log: l });
    for _ in 0..5 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    (page, cx, log)
}

#[gpui_kit::test]
fn nothing_is_drawn_when_nothing_runs_and_a_press_on_the_pill_is_heard(cx: &mut TestAppContext) {
    let (page, cx, log) = open(true, IslandCounts { running: 1, needs: 0, done: 0 }, cx);
    let pill = cx.debug_bounds("sessions-island").expect("the pill");
    assert_eq!(f32::from(pill.size.height), 28.);
    cx.simulate_click(pill.center(), Modifiers::default());
    assert_eq!(*log.borrow(), vec!["press"]);
    page.update(cx, |p, cx| {
        p.counts = IslandCounts::default();
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("sessions-island").is_none());
}

#[gpui_kit::test]
fn the_pill_grows_to_the_content_on_the_spring_when_a_state_appears(cx: &mut TestAppContext) {
    motion::clock::freeze();
    let (page, cx, _) = open(false, IslandCounts { running: 1, needs: 0, done: 0 }, cx);
    for _ in 0..60 {
        motion::clock::advance(Duration::from_millis(30));
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    let small = f32::from(cx.debug_bounds("sessions-island").unwrap().size.width);
    page.update(cx, |p, cx| {
        p.counts = IslandCounts { running: 1, needs: 2, done: 3 };
        cx.notify();
    });
    for _ in 0..3 {
        motion::clock::advance(Duration::from_millis(30));
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    let mid = f32::from(cx.debug_bounds("sessions-island").unwrap().size.width);
    for _ in 0..90 {
        motion::clock::advance(Duration::from_millis(30));
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    let full = f32::from(cx.debug_bounds("sessions-island").unwrap().size.width);
    assert!(mid > small && mid < full, "on its way: {small} then {mid} then {full}");
}

/// The island wears the title bar's look: a `card_strong` chip with the foreground on it, no shadow, 28px tall.
#[test]
fn the_island_is_a_quiet_chip_whose_words_read_in_every_theme() {
    use crate::theme::{TEXT_CONTRAST, contrast};
    for theme in crate::themes::all() {
        let (fill, ink) = colors(theme);
        assert_eq!(fill, theme.card_strong, "{}", theme.name);
        assert_eq!(ink, theme.foreground);
        assert!(contrast(ink, fill) >= TEXT_CONTRAST, "{}: {:.2}", theme.name, contrast(ink, fill));
    }
    assert_eq!(HEIGHT, 28.);
}
