use super::{
    Activation, Badge, Connection, Folds, ListMode, Location, Nav, ProjectData, Row,
    SessionData, Step, activate, held_order, key_of, position_of, priority_rows, rows, since,
    sorted, step,
};
use crate::{
    agent_look::{AgentLook, Mark, PhaseLabels},
    session_status::{Need, SessionStatus},
    sprite::Strip,
};

fn look() -> AgentLook {
    let strip = Strip { path: "icons/bot.svg", bytes: b"", frames: 1, frame_ms: 1000, loops: false };
    let color = gpui_kit::Hsla::default();
    AgentLook { mark: Mark { working: strip, orbiting: strip, color, icon_frame: 0 }, message: color, glimmer: color, labels: PhaseLabels::default() }
}

fn session(id: &str, status: SessionStatus, active_at: u64) -> SessionData {
    SessionData { archived: false, in_panel: false, provider: None, id: id.to_string().into(), title: format!("Title {id}").into(), look: look(), status, active_at }
}

fn project(id: &str, sessions: Vec<SessionData>) -> ProjectData {
    ProjectData {
        id: id.to_string().into(),
        name: id.to_string().into(),
        location: Location::Local,
        connection: Connection::Connected,
        sessions,
        pulls_unavailable: None,
        badge: Default::default(),
    }
}

fn idle(id: &str, at: u64) -> SessionData {
    session(id, SessionStatus::Idle, at)
}

fn ids(project: &ProjectData, order: &[usize]) -> Vec<String> {
    order.iter().map(|&i| project.sessions[i].id.to_string()).collect()
}

#[test]
fn sessions_that_need_you_come_first_and_the_rest_by_latest_activity() {
    let p = project(
        "p",
        vec![
            idle("old", 10),
            session("asks", SessionStatus::NeedsYou(Need::Question), 5),
            idle("new", 90),
            session("approve", SessionStatus::NeedsYou(Need::Approval), 50),
            session("working", SessionStatus::Working, 70),
        ],
    );
    assert_eq!(ids(&p, &sorted(&p.sessions)), ["approve", "asks", "new", "working", "old"]);
}

#[test]
fn equal_activity_keeps_the_order_the_app_gave() {
    let p = project("p", vec![idle("a", 5), idle("b", 5), idle("c", 5)]);
    assert_eq!(ids(&p, &sorted(&p.sessions)), ["a", "b", "c"]);
}

#[test]
fn a_project_shows_five_sessions_then_a_fold_with_the_count_of_the_rest() {
    let sessions: Vec<_> = (0..12).map(|i| idle(&format!("s{i}"), 100 - i)).collect();
    let projects = [project("p", sessions)];
    let list = rows(&projects, &Folds::default());
    assert_eq!(
        list.len(),
        1 + 5 + 1,
        "the header, five sessions, the fold"
    );
    assert_eq!(
        list[6],
        Row::Older {
            project: 0,
            hidden: 7,
            open: false
        }
    );
    let mut folds = Folds::default();
    folds.toggle_older(&"p".into());
    let open = rows(&projects, &folds);
    assert_eq!(open.len(), 1 + 12 + 1);
    assert_eq!(
        open[13],
        Row::Older {
            project: 0,
            hidden: 7,
            open: true
        },
        "the same row says Show fewer"
    );
}

#[test]
fn six_sessions_fold_one_and_five_fold_none() {
    let five = [project("p", (0..5).map(|i| idle(&format!("s{i}"), i)).collect())];
    assert!(!rows(&five, &Folds::default()).iter().any(|r| matches!(r, Row::Older { .. })));
    let six = [project("p", (0..6).map(|i| idle(&format!("s{i}"), i)).collect())];
    assert!(rows(&six, &Folds::default()).contains(&Row::Older { project: 0, hidden: 1, open: false }));
}

#[test]
fn a_session_with_news_is_never_folded_away() {
    // Ten idle sessions, all more recent than the one that finished.
    let mut sessions: Vec<_> = (0..10).map(|i| idle(&format!("s{i}"), 1000 + i)).collect();
    sessions.push(session("news", SessionStatus::Finished, 1));
    let projects = [project("p", sessions)];
    let list = rows(&projects, &Folds::default());
    let shown: Vec<_> = list.iter().filter_map(|r| if let Row::Session { session, .. } = r { Some(*session) } else { None }).collect();
    assert!(shown.contains(&10), "the finished session shows, though it is the oldest");
    assert!(!list.iter().any(|r| matches!(r, Row::Older { .. })), "nothing is left to fold");
}

#[test]
fn a_collapsed_project_shows_only_its_header_and_an_empty_one_offers_a_new_session() {
    let projects = [project("a", vec![idle("x", 1)]), project("b", vec![])];
    let mut folds = Folds::default();
    folds.set_collapsed(&"a".into(), true);
    assert_eq!(
        rows(&projects, &folds),
        [
            Row::Project { project: 0 },
            Row::Project { project: 1 },
            Row::Empty { project: 1 },
        ]
    );
    folds.set_collapsed(&"a".into(), false);
    assert_eq!(rows(&projects, &folds).len(), 4);
}

#[test]
fn a_collapsed_project_with_sessions_that_need_you_still_shows_only_its_header() {
    let projects = [project("a", vec![session("x", SessionStatus::NeedsYou(Need::Approval), 1)])];
    let mut folds = Folds::default();
    folds.set_collapsed(&"a".into(), true);
    assert_eq!(rows(&projects, &folds), [Row::Project { project: 0 }]);
}

#[test]
fn no_projects_no_rows() {
    assert!(rows(&[], &Folds::default()).is_empty());
}

fn press(rows_: &[Row], collapsed: &[usize], at: Option<usize>, nav: Nav) -> Step {
    step(rows_, |p| collapsed.contains(&p), at, nav)
}

fn sample() -> Vec<Row> {
    vec![
        Row::Project { project: 0 },
        Row::Session { project: 0, session: 0 },
        Row::Session { project: 0, session: 1 },
        Row::Project { project: 1 },
        Row::Empty { project: 1 },
    ]
}

#[test]
fn up_and_down_move_one_row_and_stop_at_the_ends() {
    let r = sample();
    assert_eq!(press(&r, &[], Some(1), Nav::Down).select, Some(2));
    assert_eq!(press(&r, &[], Some(1), Nav::Up).select, Some(0));
    assert_eq!(press(&r, &[], Some(0), Nav::Up).select, Some(0));
    assert_eq!(press(&r, &[], Some(4), Nav::Down).select, Some(4));
    assert_eq!(press(&r, &[], Some(2), Nav::First).select, Some(0));
    assert_eq!(press(&r, &[], Some(2), Nav::Last).select, Some(4));
}

#[test]
fn with_nothing_selected_down_takes_the_first_row_and_up_the_last() {
    let r = sample();
    assert_eq!(press(&r, &[], None, Nav::Down).select, Some(0));
    assert_eq!(press(&r, &[], None, Nav::Up).select, Some(4));
    assert_eq!(press(&r, &[], Some(99), Nav::Down).select, Some(0), "a selection past the rows is no selection");
    assert_eq!(press(&[], &[], None, Nav::Down), Step::default());
}

#[test]
fn left_on_a_session_goes_to_its_project_and_on_an_open_project_folds_it() {
    let r = sample();
    assert_eq!(press(&r, &[], Some(2), Nav::Left), Step { select: Some(0), fold: None });
    assert_eq!(press(&r, &[], Some(0), Nav::Left), Step { select: Some(0), fold: Some((0, true)) });
    assert_eq!(press(&r, &[0], Some(0), Nav::Left), Step { select: Some(0), fold: None }, "already folded");
    assert_eq!(press(&r, &[], Some(4), Nav::Left).select, Some(3), "an empty row goes to its project too");
}

#[test]
fn right_on_a_folded_project_unfolds_it_and_on_an_open_one_goes_in() {
    let r = sample();
    assert_eq!(press(&r, &[0], Some(0), Nav::Right), Step { select: Some(0), fold: Some((0, false)) });
    assert_eq!(press(&r, &[], Some(0), Nav::Right), Step { select: Some(1), fold: None });
    assert_eq!(press(&r, &[], Some(1), Nav::Right), Step { select: Some(1), fold: None }, "a session has nothing inside");
}

#[test]
fn enter_opens_a_session_folds_a_project_and_starts_a_session_in_an_empty_one() {
    assert_eq!(activate(Row::Session { project: 2, session: 3 }), Activation::OpenSession { project: 2, session: 3 });
    assert_eq!(activate(Row::Project { project: 1 }), Activation::ToggleProject(1));
    assert_eq!(activate(Row::Older { project: 1, hidden: 4, open: false }), Activation::ToggleOlder(1));
    assert_eq!(activate(Row::Empty { project: 5 }), Activation::NewSession(5));
}

#[test]
fn a_selection_follows_its_row_when_the_rows_reorder() {
    let mut projects = [project("p", vec![idle("a", 1), idle("b", 2)])];
    let before = rows(&projects, &Folds::default());
    let key = key_of(&projects, before[2]);
    assert_eq!(position_of(&projects, &before, &key), Some(2));
    // "a" becomes the most recent: it moves to the top of the project.
    projects[0].sessions[0].active_at = 99;
    let after = rows(&projects, &Folds::default());
    assert_eq!(position_of(&projects, &after, &key), Some(1));
    projects[0].sessions.remove(0);
    let gone = rows(&projects, &Folds::default());
    assert_eq!(position_of(&projects, &gone, &key), None);
}

#[test]
fn a_session_that_starts_needing_you_moves_to_the_top_of_its_project() {
    let mut projects = [project("p", vec![idle("a", 9), idle("b", 5), idle("c", 1)])];
    let top = |projects: &[ProjectData]| match rows(projects, &Folds::default())[1] {
        Row::Session { session, .. } => projects[0].sessions[session].id.to_string(),
        other => panic!("{other:?}"),
    };
    assert_eq!(top(&projects), "a");
    projects[0].sessions[2].status = SessionStatus::NeedsYou(Need::Approval);
    assert_eq!(top(&projects), "c");
}

#[test]
fn time_since_is_one_short_unit() {
    let now = 10_000_000;
    for (then, said) in [
        (now, "now"),
        (now - 30, "now"),
        (now - 45, "1m"),
        (now - 120, "2m"),
        (now - 3599, "60m"),
        (now - 3600, "1h"),
        (now - 7 * 3600, "7h"),
        (now - 86_400, "1d"),
        (now - 13 * 86_400, "13d"),
        (now - 14 * 86_400, "2w"),
        (now + 500, "now"),
    ] {
        assert_eq!(since(now, then), said);
    }
}
/// While the pointer holds the list, the rows keep the order they had: a session that turns busy stays
/// where it was, one that leaves gives its place to one that arrives (a past session opening), and
/// others that arrive go last.
#[test]
fn a_held_list_keeps_its_order() {
    let now = vec![idle("c", 30), idle("a", 10), idle("b", 5)];
    let names = |order: Vec<usize>| order.into_iter().map(|i| now[i].id.to_string()).collect::<Vec<_>>();
    let before: Vec<gpui_kit::SharedString> = ["a", "b", "c"].map(gpui_kit::SharedString::from).to_vec();
    assert_eq!(names(held_order(&before, &now)), ["a", "b", "c"], "c turned busy and stays third");
    let now = vec![idle("a", 10), idle("c", 30), idle("x", 20), idle("y", 1)];
    let names = |order: Vec<usize>| order.into_iter().map(|i| now[i].id.to_string()).collect::<Vec<_>>();
    assert_eq!(names(held_order(&before, &now)), ["a", "x", "c", "y"], "x takes b's place, y goes last");
}

#[test]
fn a_project_section_ends_with_its_last_session_and_holds_no_tasks() {
    let list = [project("p", vec![idle("a", 2), idle("b", 1)])];
    assert_eq!(
        rows(&list, &Folds::default()).last(),
        Some(&Row::Session { project: 0, session: 1 }),
        "tasks live in the Tasks view, not under the sessions"
    );
}

fn status_session(id: &str, status: SessionStatus, at: u64) -> SessionData {
    SessionData { status, ..session(id, SessionStatus::Idle, at) }
}

fn two_projects(a: Vec<SessionData>, b: Vec<SessionData>) -> Vec<ProjectData> {
    let project = |name: &str, sessions: Vec<SessionData>| ProjectData {
        id: name.to_string().into(),
        name: name.to_string().into(),
        location: Location::Local,
        connection: Connection::Connected,
        sessions,
        pulls_unavailable: None,
        badge: Badge::default(),
    };
    vec![project("a", a), project("b", b)]
}

fn at(rows: &[Row], projects: &[ProjectData]) -> Vec<String> {
    rows.iter()
        .map(|r| match *r {
            Row::Section { section, count } => format!("# {} {count}", section.words()),
            Row::Session { project, session } => projects[project].sessions[session].id.to_string(),
            Row::MoreEarlier { hidden, open } => format!("more {hidden} {open}"),
            other => format!("{other:?}"),
        })
        .collect()
}

/// The priority list: the sessions that need the reader first, across projects, each section under its heading, the
/// empty ones left out.
#[test]
fn the_priority_list_puts_what_needs_you_first_across_projects() {
    let projects = two_projects(
        vec![status_session("a-work", SessionStatus::Working, 50), status_session("a-ask", SessionStatus::NeedsYou(Need::Question), 40), status_session("a-old", SessionStatus::Idle, 10)],
        vec![status_session("b-done", SessionStatus::Finished, 60), status_session("b-ask", SessionStatus::NeedsYou(Need::Approval), 20), status_session("b-fail", SessionStatus::Failed("x".into()), 30)],
    );
    let rows = priority_rows(&projects, false, crate::sidebar_layout::EARLIER_SHOWN);
    assert_eq!(
        at(&rows, &projects),
        ["# Needs you 3", "b-ask", "b-fail", "a-ask", "# Finished 1", "b-done", "# Working 1", "a-work", "# Earlier 1", "a-old"],
        "the one waiting longest is first, then each section in its place"
    );
    let none = two_projects(vec![status_session("x", SessionStatus::Working, 1)], vec![]);
    assert_eq!(at(&priority_rows(&none, false, crate::sidebar_layout::EARLIER_SHOWN), &none), ["# Working 1", "x"], "a section with nothing in it has no heading");
}

/// Within a section the order depends on what the session is, not on when the agent last wrote: working sessions
/// keep the order they began in, however their last activity moves.
#[test]
fn working_sessions_keep_their_order_when_their_activity_moves() {
    let mut projects = two_projects(vec![status_session("s1", SessionStatus::Working, 10), status_session("s2", SessionStatus::Working, 20)], vec![]);
    let before = at(&priority_rows(&projects, false, crate::sidebar_layout::EARLIER_SHOWN), &projects);
    projects[0].sessions[0].active_at = 999;
    assert_eq!(at(&priority_rows(&projects, false, crate::sidebar_layout::EARLIER_SHOWN), &projects), before);
}

/// Earlier sessions show eight, then "Show more"; open, all of them.
#[test]
fn the_earlier_section_shows_eight_then_a_show_more_row() {
    let sessions: Vec<SessionData> = (0..11).map(|i| status_session(&format!("e{i:02}"), SessionStatus::Idle, 100 - i as u64)).collect();
    let projects = two_projects(sessions, vec![]);
    let closed = at(&priority_rows(&projects, false, crate::sidebar_layout::EARLIER_SHOWN), &projects);
    assert_eq!(closed.len(), 1 + 8 + 1);
    assert_eq!(closed[0], "# Earlier 11");
    assert_eq!(closed.last().unwrap(), "more 3 false");
    let open = at(&priority_rows(&projects, true, crate::sidebar_layout::EARLIER_SHOWN), &projects);
    assert_eq!(open.len(), 1 + 11 + 1);
    assert_eq!(open.last().unwrap(), "more 3 true");
}

#[test]
fn a_list_mode_is_kept_by_its_key() {
    assert_eq!(ListMode::from_key(Some("priority")), ListMode::Priority);
    assert_eq!(ListMode::from_key(Some("whatever")), ListMode::Projects);
    assert_eq!(ListMode::from_key(None), ListMode::Projects);
    assert_eq!(ListMode::Priority.key(), "priority");
}
