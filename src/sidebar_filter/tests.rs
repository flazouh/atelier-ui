use super::*;
use crate::{
    agent_look::AgentLook,
    session_status::Need,
    sidebar_model::{Connection, Location, ProjectData},
};

fn session(title: &str, status: SessionStatus, archived: bool) -> SessionData {
    SessionData { in_panel: false, provider: None, id: title.into(), title: title.into(), look: AgentLook::neutral(&crate::theme::Theme::light()), status, active_at: 1, archived }
}

fn project(sessions: Vec<SessionData>) -> ProjectData {
    ProjectData {
        id: "p".into(),
        name: "p".into(),
        location: Location::Local,
        connection: Connection::Connected,
        sessions,
        pulls_unavailable: None,
        badge: crate::sidebar_model::Badge {
            label: "P".into(),
            color: 0,
            icon: None,
        },
    }
}

fn titles(projects: &[ProjectData]) -> Vec<String> {
    projects.iter().flat_map(|p| p.sessions.iter().map(|s| s.title.to_string())).collect()
}

fn sample() -> Vec<ProjectData> {
    vec![project(vec![
        session("fix login", SessionStatus::Working, false),
        session("write docs", SessionStatus::NeedsYou(Need::Question), false),
        session("old idea", SessionStatus::Idle, true),
        session("ship it", SessionStatus::Finished, false),
        session("quiet", SessionStatus::Idle, false),
    ])]
}

/// Each filter keeps what it names, and an archived session shows only under Archived and All.
#[test]
fn each_filter_keeps_what_it_names() {
    let p = sample();
    let by = |f| titles(&narrow(&p, f));
    assert_eq!(by(SessionFilter::Active), ["fix login", "write docs", "ship it", "quiet"]);
    assert_eq!(by(SessionFilter::NeedsYou), ["write docs", "ship it"]);
    assert_eq!(by(SessionFilter::Working), ["fix login"]);
    assert_eq!(by(SessionFilter::Archived), ["old idea"]);
    assert_eq!(by(SessionFilter::All).len(), 5);
}

/// A project stays on the list with none of its sessions left.
#[test]
fn a_project_stays_with_no_session_left() {
    let narrowed = narrow(&[project(vec![session("quiet", SessionStatus::Idle, false)])], SessionFilter::Working);
    assert_eq!(narrowed.len(), 1);
    assert!(narrowed[0].sessions.is_empty());
}

#[test]
fn the_hidden_count_and_the_words() {
    let p = sample();
    assert_eq!(hidden_by(&p, SessionFilter::Active), 1);
    assert_eq!(hidden_by(&p, SessionFilter::All), 0);
    assert_eq!(describe(SessionFilter::Active), "Filter sessions");
    assert_eq!(describe(SessionFilter::Archived), "Showing: Archived");
}
