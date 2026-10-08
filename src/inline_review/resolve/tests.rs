use super::*;
use std::time::Duration;

fn ms(n: u128) -> Duration {
    Duration::from_millis(n as u64)
}

fn fade_after(n: u128) -> Option<f32> {
    let start = Instant::now();
    Resolve::at("h", Decision::Accept, start).fade(start + ms(n), false)
}

/// A resolve whose edit ran at `edit` and deleted rows 4..7, read `n` ms after the edit.
fn gap_after(n: u128) -> Option<(usize, f32)> {
    let start = Instant::now();
    let edit = start + duration::RESOLVE_FADE;
    let mut resolve = Resolve::at("h", Decision::Accept, start);
    resolve.edited_at(4..7, edit);
    resolve.gap(edit + ms(n), false)
}

#[test]
fn the_rows_are_whole_when_the_decision_is_made() {
    assert_eq!(fade_after(0), Some(1.));
}

#[test]
fn the_rows_thin_as_the_fade_runs() {
    let early = fade_after(20).expect("still fading");
    let late = fade_after(80).expect("still fading");
    assert!(1. > early && early > late && late > 0., "{early} then {late}");
}

#[test]
fn the_fade_ends_at_the_fade_duration() {
    assert!(fade_after(duration::RESOLVE_FADE.as_millis() - 1).is_some());
    assert_eq!(fade_after(duration::RESOLVE_FADE.as_millis()), None, "the edit runs now");
}

#[test]
fn there_is_no_gap_before_the_edit() {
    let start = Instant::now();
    assert_eq!(Resolve::at("h", Decision::Accept, start).gap(start + ms(500), false), None);
}

#[test]
fn the_gap_starts_as_tall_as_the_deleted_rows_and_sits_where_they_were() {
    assert_eq!(gap_after(0), Some((4, 3.)));
}

#[test]
fn the_gap_shrinks_as_the_collapse_runs() {
    let (_, early) = gap_after(30).expect("still closing");
    let (_, late) = gap_after(150).expect("still closing");
    assert!(3. > early && early > late && late > 0., "{early} then {late}");
}

#[test]
fn the_collapse_ends_at_the_resolve_duration() {
    assert!(gap_after(duration::RESOLVE.as_millis() - 1).is_some());
    assert_eq!(gap_after(duration::RESOLVE.as_millis()), None);
}

#[test]
fn a_hunk_that_deleted_nothing_has_no_gap() {
    let start = Instant::now();
    let mut resolve = Resolve::at("h", Decision::Accept, start);
    resolve.edited_at(4..4, start);
    assert_eq!(resolve.gap(start, false), None);
    assert!(resolve.is_over(start, false));
}

#[test]
fn the_fade_stops_once_the_edit_ran() {
    let start = Instant::now();
    let mut resolve = Resolve::at("h", Decision::Accept, start);
    resolve.edited_at(4..7, start);
    assert_eq!(resolve.fade(start, false), None, "the rows it would fade are gone");
}

#[test]
fn the_resolve_is_over_only_once_the_gap_has_closed() {
    let start = Instant::now();
    let mut resolve = Resolve::at("h", Decision::Accept, start);
    assert!(!resolve.is_over(start + ms(1000), false), "the edit has not run");
    resolve.edited_at(4..7, start);
    assert!(!resolve.is_over(start, false));
    assert!(resolve.is_over(start + duration::RESOLVE, false));
}

#[test]
fn reduce_motion_has_no_fade_and_no_gap() {
    let start = Instant::now();
    let mut resolve = Resolve::at("h", Decision::Reject, start);
    assert_eq!(resolve.fade(start, true), None);
    resolve.edited_at(4..7, start);
    assert_eq!(resolve.gap(start, true), None);
}
