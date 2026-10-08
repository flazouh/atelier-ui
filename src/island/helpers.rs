use gpui_kit::SharedString;

use crate::{
    session_status::{Need, SessionStatus},
    sidebar_model::ProjectData,
};
use super::structs::IslandCounts;

/// The chip's fill and the words on it.
pub fn colors(theme: &crate::theme::Theme) -> (gpui_kit::Hsla, gpui_kit::Hsla) {
    (theme.card_strong, theme.foreground)
}

/// The counts over every session of every project.
pub fn counts_of(projects: &[ProjectData]) -> IslandCounts {
    let mut counts = IslandCounts::default();
    for session in projects.iter().flat_map(|p| &p.sessions) {
        match session.status {
            SessionStatus::Working => counts.running += 1,
            SessionStatus::NeedsYou(_) => counts.needs += 1,
            SessionStatus::Finished => counts.done += 1,
            _ => {}
        }
    }
    counts
}

/// The session that needs the reader most: one waiting for a yes or no, then one with a question, then one
/// that finished unseen, then one that works. Among equals, the one that did something last. Its project
/// and its own id.
pub fn most_urgent(projects: &[ProjectData]) -> Option<(SharedString, SharedString)> {
    let rank = |status: &SessionStatus| match status {
        SessionStatus::NeedsYou(Need::Approval) => Some(0),
        SessionStatus::NeedsYou(Need::Question) => Some(1),
        SessionStatus::Finished => Some(2),
        SessionStatus::Working => Some(3),
        _ => None,
    };
    projects
        .iter()
        .flat_map(|p| p.sessions.iter().map(move |s| (p, s)))
        .filter_map(|(p, s)| rank(&s.status).map(|r| (r, std::cmp::Reverse(s.active_at), p, s)))
        .min_by_key(|(r, at, ..)| (*r, *at))
        .map(|(_, _, p, s)| (p.id.clone(), s.id.clone()))
}
