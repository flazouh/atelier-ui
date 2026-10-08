use super::*;

#[test]
fn the_stroke_is_nine_percent_of_the_size_and_never_under_two() {
    assert!((stroke_width(32.) - 2.88).abs() < 1e-4);
    assert_eq!(stroke_width(12.), 2.);
    assert_eq!(stroke_width(14.), 2.);
    assert!((stroke_width(100.) - 9.).abs() < 1e-4);
}

#[test]
fn it_turns_once_a_second_at_a_steady_speed() {
    assert_eq!(turn_at(0), 0.);
    assert!((turn_at(250) - std::f32::consts::FRAC_PI_2).abs() < 1e-4);
    assert!((turn_at(500) - std::f32::consts::PI).abs() < 1e-4);
    assert_eq!(turn_at(1000), 0., "a full turn is the start again");
}

#[test]
fn the_reduced_pulse_goes_from_full_to_40_percent_and_back_over_1400_ms() {
    assert!((pulse_at(0) - 1.).abs() < 1e-4);
    assert!((pulse_at(700) - PULSE_LOW).abs() < 1e-3);
    assert!((pulse_at(1399) - 1.).abs() < 0.01);
    assert!(pulse_at(350) < 1. && pulse_at(350) > PULSE_LOW);
}

#[test]
fn a_quarter_arc_runs_from_the_top_to_the_right() {
    let points = arc((10., 10.), 5., 0., std::f32::consts::FRAC_PI_2, 8);
    let (first, last) = (points[0], points[8]);
    assert!(
        (first.0 - 10.).abs() < 1e-4 && (first.1 - 5.).abs() < 1e-4,
        "the top: {first:?}"
    );
    assert!(
        (last.0 - 15.).abs() < 1e-4 && (last.1 - 10.).abs() < 1e-4,
        "the right: {last:?}"
    );
    assert_eq!(points.len(), 9);
}
