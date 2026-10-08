use super::*;
use gpui_kit::{point, px};

const MAX: f32 = 100.;

fn used(offset_y: f32, delta_y: f32) -> bool {
    used_the_wheel(
        point(px(0.), px(offset_y)),
        point(px(0.), px(MAX)),
        point(px(0.), px(delta_y)),
    )
}

#[test]
fn a_box_in_the_middle_keeps_the_wheel_either_way() {
    assert!(used(-50., -10.));
    assert!(used(-50., 10.));
}

#[test]
fn at_the_start_it_hands_on_a_wheel_that_goes_back() {
    // The box's own listener added +10 to an offset of 0, past its start.
    assert!(!used(10., 10.));
    assert!(used(-10., -10.));
}

#[test]
fn at_the_end_it_hands_on_a_wheel_that_goes_further() {
    assert!(!used(-110., -10.));
    assert!(used(-90., 10.));
}

#[test]
fn a_box_with_nothing_to_scroll_never_keeps_the_wheel() {
    assert!(!used_the_wheel(
        point(px(0.), px(0.)),
        point(px(0.), px(0.)),
        point(px(0.), px(-10.))
    ));
}

#[test]
fn a_sideways_wheel_counts_on_the_sideways_axis() {
    let max = point(px(80.), px(0.));
    assert!(used_the_wheel(
        point(px(-20.), px(0.)),
        max,
        point(px(-5.), px(0.))
    ));
    assert!(!used_the_wheel(
        point(px(5.), px(0.)),
        max,
        point(px(5.), px(0.))
    ));
}

#[test]
fn a_box_at_its_end_keeps_following() {
    assert!(follows(true, 0.));
    assert!(follows(true, FOLLOW_REACH));
}

#[test]
fn a_reader_who_scrolls_up_past_the_reach_lets_go() {
    assert!(!follows(true, FOLLOW_REACH + 1.));
}

#[test]
fn it_takes_hold_again_only_at_the_end() {
    assert!(!follows(false, FOLLOW_REACH));
    assert!(!follows(false, 20.));
    assert!(follows(false, 0.5));
}
