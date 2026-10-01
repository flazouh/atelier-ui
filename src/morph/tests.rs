use std::time::{Duration, Instant};

use super::*;

fn approx(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-6, "expected {a} to equal {b}");
}

#[test]
fn at_the_start_the_old_child_is_whole_and_the_new_one_waits_below() {
    let f = frame(0.);
    assert_eq!((f.old_opacity, f.old_y, f.new_opacity, f.new_y), (1., 0., 0., MORPH_RISE));
}

#[test]
fn halfway_both_are_half_faded_and_both_rise() {
    let f = frame(0.5);
    assert_eq!((f.old_opacity, f.new_opacity), (0.5, 0.5));
    assert_eq!((f.old_y, f.new_y), (-MORPH_RISE / 2., MORPH_RISE / 2.));
}

#[test]
fn at_the_end_the_old_child_is_gone_and_the_new_one_has_settled() {
    let f = frame(1.);
    assert_eq!((f.old_opacity, f.old_y, f.new_opacity, f.new_y), (0., -MORPH_RISE, 1., 0.));
}

// A key change while settled (no exit running) starts a fresh exit from full opacity, and a fresh
// enter from nothing.
#[test]
fn a_key_change_at_p0_starts_both_channels_fresh() {
    let now = Instant::now();
    let (exit, enter) = on_key_change(None, false, now);
    approx(exit.value_at(now), 0.);
    approx(enter.value_at(now), 0.);
}

// A key change while a morph is already running must not touch the channel driving the child that
// is fading out: its value right before and right after the change is identical, so it never jumps.
#[test]
fn a_key_change_at_p0_5_does_not_jump_the_outgoing_child() {
    let now = Instant::now();
    let mut running_exit = Channel::new(0.);
    running_exit.animate_at(1., morph_curve(), 0., false, now - Duration::from_millis(90));
    let before = running_exit.value_at(now);
    assert!(before > 0. && before < 1., "expected a value mid-flight, got {before}");

    let (exit, enter) = on_key_change(Some(running_exit), false, now);
    let after = exit.value_at(now);

    approx(before, after);
    approx(enter.value_at(now), 0.);
}

// Three key changes 33ms apart (well within the 180ms morph) each restart the incoming child, but
// never disturb the one still fading out from the very first change.
#[test]
fn three_changes_in_100ms_keep_the_exit_continuous() {
    let t0 = Instant::now();
    let mut exit = Channel::new(0.);
    exit.animate_at(1., morph_curve(), 0., false, t0);
    let mut enter = Channel::new(0.);
    enter.animate_at(1., morph_curve(), 0., false, t0);

    for step in 1..=3 {
        let now = t0 + Duration::from_millis(step * 33);
        let before = exit.value_at(now);
        let (next_exit, next_enter) = on_key_change(Some(exit), false, now);
        approx(next_exit.value_at(now), before);
        approx(next_enter.value_at(now), 0.);
        exit = next_exit;
        enter = next_enter;
    }
    // The exit channel is still the same one from t0, so it keeps advancing toward 1 on its own
    // clock regardless of how many times the incoming child restarted.
    let later = t0 + Duration::from_millis(180);
    approx(exit.value_at(later), 1.);
    let _ = enter;
}
