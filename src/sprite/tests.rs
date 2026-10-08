use super::*;

/// A looping strip of 9 frames at 90ms, like an agent's thinking mark.
const LOOP: Strip = Strip { path: "test/loop.svg", bytes: b"", frames: 9, frame_ms: 90, loops: true };
/// A slow loop: 16 frames at 600ms.
const SLOW: Strip = Strip { path: "test/slow.svg", bytes: b"", frames: 16, frame_ms: 600, loops: true };
/// A one-shot strip of 6 frames at 70ms, like an entrance.
const ONCE: Strip = Strip { path: "test/once.svg", bytes: b"", frames: 6, frame_ms: 70, loops: false };

#[test]
fn frame_zero_shows_at_the_start() {
    for strip in [LOOP, SLOW, ONCE] {
        assert_eq!(strip.frame_at(0), 0, "{}", strip.path);
    }
}

#[test]
fn each_frame_shows_for_one_period() {
    assert_eq!(LOOP.frame_at(89), 0);
    assert_eq!(LOOP.frame_at(90), 1);
    assert_eq!(LOOP.frame_at(8 * 90 + 89), 8);
}

#[test]
fn a_looping_strip_wraps_after_its_last_frame() {
    // 9 frames of 90ms, so one cycle is 810ms.
    assert_eq!(LOOP.frame_at(810), 0);
    assert_eq!(LOOP.frame_at(810 + 90), 1);
    assert_eq!(SLOW.frame_at(16 * 600 + 600), 1);
}

#[test]
fn a_one_shot_strip_holds_its_last_frame() {
    assert_eq!(ONCE.frame_at(5 * 70), 5);
    assert_eq!(ONCE.frame_at(10_000), 5);
}

#[test]
fn the_next_frame_is_one_period_boundary_away() {
    assert_eq!(LOOP.next_frame_in(0), Some(90));
    assert_eq!(LOOP.next_frame_in(100), Some(80));
    assert_eq!(LOOP.next_frame_in(100_000), Some(90 - 100_000 % 90));
}

#[test]
fn a_finished_one_shot_asks_for_no_more_frames() {
    assert_eq!(ONCE.next_frame_in(4 * 70), Some(70));
    assert_eq!(ONCE.next_frame_in(5 * 70), None);
}

#[test]
fn a_strip_change_restarts() {
    assert!(restarts(&LOOP, false, &ONCE));
}

#[test]
fn resuming_from_still_restarts_even_with_the_same_strip() {
    // `.playing(false)` then `.playing(true)` on the same strip must restart at frame 0, not resume from
    // wherever `start` was left.
    assert!(restarts(&LOOP, true, &LOOP));
}

#[test]
fn playing_continuously_does_not_restart() {
    assert!(!restarts(&LOOP, false, &LOOP));
}

#[test]
fn native_size_reads_the_view_box() {
    assert_eq!(native_size(br#"<svg viewBox="0 0 100 601">"#), (100., 601.));
    assert_eq!(native_size(br#"<svg viewBox="0 0 101 601">"#), (101., 601.));
}

#[test]
fn native_size_falls_back_to_square_when_missing_or_malformed() {
    assert_eq!(native_size(b"<svg>"), (100., 100.));
    assert_eq!(native_size(br#"<svg viewBox="oops">"#), (100., 100.));
}

#[test]
fn a_square_view_box_gives_back_the_requested_size_exactly() {
    // viewBox 100x900, 9 frames.
    assert_eq!(frame_height((100., 900.), 9, 18.), 18.);
}

#[test]
fn a_strip_a_unit_off_keeps_its_frames_from_drifting_a_whole_pixel() {
    // viewBox 100x601, 6 frames. A naive `size` assumption drifts 5 * (601/600 - 1) * size by the last
    // frame; the real per-frame height keeps that from accumulating.
    let tall = frame_height((100., 601.), 6, 18.);
    assert!((tall - 18.03).abs() < 0.01, "{tall}");
    // viewBox 101x601, 6 frames.
    let wide = frame_height((101., 601.), 6, 18.);
    assert!((wide - 17.85).abs() < 0.01, "{wide}");
}

#[test]
fn a_frame_is_not_scaled_by_the_zoom_a_second_time() {
    crate::scale::set_zoom(1.2);
    let (size, frame) = frame_box(crate::scale::px(18.), (100., 900.), 9);
    crate::scale::set_zoom(1.);
    assert_eq!(f32::from(size), 22., "18 design pixels at 1.2 are 21.6, kept whole");
    assert_eq!(f32::from(frame), 22., "a square frame is as tall as the box, not 1.2 times taller");
}
