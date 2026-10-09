use crate::update_button::{
    consts::{RING, RING_STROKE},
    helpers::{ring_points, ring_start},
};

const C: f32 = RING / 2.;
const R: f32 = (RING - RING_STROKE) / 2.;

fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
}

#[test]
fn no_progress_draws_no_arc() {
    assert!(ring_points(0., 0.).is_empty());
}

#[test]
fn a_quarter_runs_from_twelve_to_three_oclock_clockwise() {
    let points = ring_points(0.25, 0.);
    assert!(near(points[0], (C, C - R)), "starts at 12 o'clock: {:?}", points[0]);
    assert!(near(*points.last().unwrap(), (C + R, C)), "ends at 3 o'clock");
}

#[test]
fn a_full_ring_closes_on_where_it_started() {
    let points = ring_points(1., 0.);
    assert!(near(points[0], *points.last().unwrap()));
    assert!(points.len() > 16, "enough segments to look round");
}

#[test]
fn progress_beyond_the_range_is_held_to_it() {
    assert_eq!(ring_points(1.7, 0.), ring_points(1., 0.));
    assert!(ring_points(-0.3, 0.).is_empty());
}

#[test]
fn the_spinner_turns_unless_motion_is_reduced() {
    assert_eq!(ring_start(250, true), 0.);
    assert!(ring_start(250, false) > 0.);
}
