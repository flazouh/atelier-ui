use super::{Mark, Need, SessionStatus, short_reason};

#[test]
fn every_status_has_its_words() {
    let words = |s: SessionStatus| s.words().to_string();
    assert_eq!(words(SessionStatus::Working), "Working");
    assert_eq!(
        words(SessionStatus::NeedsYou(Need::Approval)),
        "Needs approval"
    );
    assert_eq!(
        words(SessionStatus::NeedsYou(Need::Question)),
        "Asks a question"
    );
    assert_eq!(words(SessionStatus::Finished), "Finished, ready to look at");
    assert_eq!(words(SessionStatus::Idle), "Idle");
    assert_eq!(
        words(SessionStatus::Failed(
            "the process exited with code 3".into()
        )),
        "Stopped: the process exited with code 3"
    );
}

#[test]
fn each_status_draws_its_own_mark() {
    assert_eq!(SessionStatus::Working.mark(), Mark::AgentWorking);
    assert_eq!(
        SessionStatus::NeedsYou(Need::Question).mark(),
        Mark::Warning
    );
    assert_eq!(SessionStatus::Finished.mark(), Mark::AmberDot);
    assert_eq!(SessionStatus::Failed("x".into()).mark(), Mark::Danger);
    assert_eq!(SessionStatus::Idle.mark(), Mark::Idle);
}

#[test]
fn the_amber_dot_and_the_warning_belong_to_different_statuses() {
    let all = [
        SessionStatus::Working,
        SessionStatus::NeedsYou(Need::Approval),
        SessionStatus::NeedsYou(Need::Question),
        SessionStatus::Finished,
        SessionStatus::Idle,
        SessionStatus::Failed("x".into()),
    ];
    assert_eq!(all.iter().filter(|s| s.mark() == Mark::AmberDot).count(), 1);
    assert!(
        all.iter()
            .filter(|s| s.mark() == Mark::Warning)
            .all(SessionStatus::needs_you)
    );
}

#[test]
fn only_a_session_that_waits_for_the_reader_shows_its_words() {
    for (status, note) in [
        (SessionStatus::Working, false),
        (SessionStatus::NeedsYou(Need::Approval), true),
        (SessionStatus::Finished, false),
        (SessionStatus::Idle, false),
        (SessionStatus::Failed("x".into()), false),
    ] {
        assert_eq!(status.has_note(), note, "{status:?}");
    }
}

#[test]
fn a_session_with_news_is_never_folded_away_and_an_idle_one_is_muted() {
    assert!(SessionStatus::Finished.wants_attention());
    assert!(SessionStatus::NeedsYou(Need::Approval).wants_attention());
    assert!(!SessionStatus::Working.wants_attention() && !SessionStatus::Idle.wants_attention());
    assert!(!SessionStatus::Idle.title_is_ink());
    assert!(SessionStatus::Working.title_is_ink() && SessionStatus::Finished.title_is_ink());
}

#[test]
fn a_reason_is_its_first_line_cut_to_what_a_row_holds() {
    assert_eq!(short_reason("exit code 3").as_ref(), "exit code 3");
    assert_eq!(
        short_reason("\n  first line \nsecond").as_ref(),
        "first line"
    );
    assert_eq!(short_reason("").as_ref(), "");
    let long = short_reason(&"x".repeat(100));
    assert_eq!(long.chars().count(), 40);
    assert!(long.ends_with('…'));
    assert_eq!(
        short_reason("é".repeat(40).as_str()).chars().count(),
        40,
        "40 characters fit"
    );
}

#[test]
fn the_five_statuses_draw_five_different_marks_and_say_five_different_things() {
    let all = [
        SessionStatus::Working,
        SessionStatus::NeedsYou(Need::Approval),
        SessionStatus::Finished,
        SessionStatus::Idle,
        SessionStatus::Failed("x".into()),
    ];
    let marks: std::collections::HashSet<_> =
        all.iter().map(|s| format!("{:?}", s.mark())).collect();
    assert_eq!(marks.len(), 5, "no two statuses share a mark: {marks:?}");
    let words: std::collections::HashSet<_> = all.iter().map(|s| s.words().to_string()).collect();
    assert_eq!(words.len(), 5, "and each has its own words for the hover");
}
