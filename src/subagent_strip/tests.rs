use std::time::{Duration, Instant};

use super::*;
use crate::agent_look::AgentLook;

fn row(id: &'static str) -> SubagentRow {
    SubagentRow::new(id, AgentLook::neutral(&crate::theme::Theme::dark()), id, "task")
}

fn ids(state: &StripState, now: Instant) -> Vec<(String, f32)> {
    state.slots(now).map(|(r, open)| (r.id().to_string(), (open * 100.).round() / 100.)).collect()
}

const SETTLE: Duration = Duration::from_secs(2);

#[test]
fn rows_on_the_first_paint_show_at_once() {
    let mut s = StripState::default();
    let t = Instant::now();
    s.sync(vec![row("a"), row("b")], t, false);
    assert_eq!(ids(&s, t), [("a".into(), 1.), ("b".into(), 1.)]);
    assert!(!s.is_moving(t));
}

#[test]
fn a_new_row_opens_from_nothing() {
    let mut s = StripState::default();
    let t = Instant::now();
    s.sync(vec![row("a")], t, false);
    s.sync(vec![row("a"), row("b")], t, false);
    assert_eq!(ids(&s, t)[1], ("b".into(), 0.));
    assert!(s.is_moving(t));
    assert_eq!(ids(&s, t + SETTLE)[1], ("b".into(), 1.));
}

#[test]
fn a_finished_row_holds_then_leaves() {
    let mut s = StripState::default();
    let t = Instant::now();
    s.sync(vec![row("a"), row("b")], t, false);
    let due = s.sync(vec![row("a").finished(Some(3)), row("b")], t, false);
    assert_eq!(due, Some(t + duration::FINISH_HOLD));
    // Still held just before its time.
    let early = t + duration::FINISH_HOLD - Duration::from_millis(1);
    s.sync(vec![row("a").finished(Some(3)), row("b")], early, false);
    assert_eq!(ids(&s, early)[0], ("a".into(), 1.));
    // Then it closes, keeping its place until it is gone.
    let late = t + duration::FINISH_HOLD;
    assert_eq!(s.sync(vec![row("a").finished(Some(3)), row("b")], late, false), None);
    assert!(s.is_moving(late));
    assert_eq!(ids(&s, late)[0].0, "a");
    s.sync(vec![row("a").finished(Some(3)), row("b")], late + SETTLE, false);
    assert_eq!(ids(&s, late + SETTLE), [("b".into(), 1.)]);
}

#[test]
fn a_row_gone_from_the_data_leaves_from_its_last_state() {
    let mut s = StripState::default();
    let t = Instant::now();
    s.sync(vec![row("a").tool("Read x"), row("b")], t, false);
    s.sync(vec![row("b")], t, false);
    let slots: Vec<_> = s.slots(t).map(|(r, _)| r.detail().to_string()).collect();
    assert_eq!(slots, ["Read x", "task"]);
    s.sync(vec![row("b")], t + SETTLE, false);
    assert_eq!(ids(&s, t + SETTLE), [("b".into(), 1.)]);
}

#[test]
fn a_row_that_comes_back_while_leaving_opens_again() {
    let mut s = StripState::default();
    let t = Instant::now();
    s.sync(vec![row("a"), row("b")], t, false);
    s.sync(vec![row("b")], t, false);
    s.sync(vec![row("a"), row("b")], t + Duration::from_millis(50), false);
    let later = t + SETTLE;
    s.sync(vec![row("a"), row("b")], later, false);
    assert_eq!(ids(&s, later), [("a".into(), 1.), ("b".into(), 1.)]);
}

#[test]
fn reduce_motion_opens_and_closes_at_once_but_still_holds() {
    let mut s = StripState::default();
    let t = Instant::now();
    s.sync(vec![row("a")], t, true);
    s.sync(vec![row("a"), row("b")], t, true);
    assert_eq!(ids(&s, t)[1], ("b".into(), 1.));
    assert!(!s.is_moving(t));
    s.sync(vec![row("a").finished(None), row("b")], t, true);
    assert_eq!(ids(&s, t).len(), 2);
    s.sync(vec![row("a").finished(None), row("b")], t + duration::FINISH_HOLD, true);
    assert_eq!(ids(&s, t + duration::FINISH_HOLD), [("b".into(), 1.)]);
}

#[test]
fn a_row_that_left_does_not_come_back_while_the_data_still_holds_it_finished() {
    let mut s = StripState::default();
    let t = Instant::now();
    s.sync(vec![row("a"), row("b")], t, false);
    let done = || vec![row("a"), row("b").finished(Some(9))];
    s.sync(done(), t, false);
    let gone = t + duration::FINISH_HOLD + SETTLE;
    s.sync(done(), t + duration::FINISH_HOLD, false);
    s.sync(done(), gone, false);
    assert_eq!(ids(&s, gone), [("a".into(), 1.)]);
    // A later frame with the same data keeps it gone.
    s.sync(done(), gone + SETTLE, false);
    assert_eq!(ids(&s, gone + SETTLE), [("a".into(), 1.)]);
    // If it runs again, it joins again.
    s.sync(vec![row("a"), row("b")], gone + SETTLE, false);
    assert_eq!(ids(&s, gone + SETTLE).len(), 2);
}
