use super::*;
use std::time::Duration;

#[test]
fn press_spring_settles_near_its_target_without_big_overshoot() {
    let mut scale = Animated::new(Spring::PRESS, 1.);
    scale.set_target(0.93);
    let mut lowest = 1f32;
    for _ in 0..200 {
        scale.step(0.001, false);
        lowest = lowest.min(scale.value());
    }
    assert!(
        (scale.value() - 0.93).abs() < 0.002,
        "value {}",
        scale.value()
    );
    assert!(lowest > 0.925, "overshoot to {lowest}");
}

#[test]
fn reduce_motion_jumps_to_the_target() {
    let mut value = Animated::new(Spring::PRESS, 0.);
    value.set_target(1.);
    assert!(!value.step(0.016, true));
    assert_eq!(value.value(), 1.);
}

#[test]
fn the_tint_spring_never_overshoots() {
    let mut slide = Animated::new(Spring::TINT, 0.);
    slide.set_target(1.);
    let mut highest = 0f32;
    for _ in 0..600 {
        slide.step(0.001, false);
        highest = highest.max(slide.value());
    }
    assert!(highest <= 1.0001, "overshoot to {highest}");
    assert!(slide.is_settled());
}

#[test]
fn cubic_bezier_hits_its_end_points() {
    assert!(cubic_bezier(ease::OUT, 0.).abs() < 1e-4);
    assert!((cubic_bezier(ease::OUT, 1.) - 1.).abs() < 1e-4);
}

#[test]
fn a_bouncy_spring_with_no_bounce_is_critically_damped() {
    let spring = Spring::bouncy(0.4, 0.);
    let critical = 2. * (spring.stiffness * spring.mass).sqrt();
    assert!((spring.damping - critical).abs() < 1e-3);
}

#[test]
fn a_bouncy_spring_overshoots_and_settles_near_its_duration() {
    let mut value = Animated::new(Spring::bouncy(0.4, 0.3), 0.);
    value.set_target(1.);
    let (mut highest, mut steps) = (0f32, 0);
    while value.step(0.001, false) {
        highest = highest.max(value.value());
        steps += 1;
    }
    assert!(highest > 1.02, "no overshoot: {highest}");
    assert!(steps < 900, "took {steps} ms");
}

#[test]
fn a_channel_waits_for_its_delay() {
    let now = Instant::now();
    let mut c = Channel::new(0.);
    c.animate_at(10., Curve::Ease(0.2, ease::OUT), 0.1, false, now);
    assert_eq!(c.value_at(now + Duration::from_millis(50)), 0.);
    assert!(c.value_at(now + Duration::from_millis(200)) > 0.);
    assert_eq!(c.value_at(now + Duration::from_millis(400)), 10.);
    assert!(!c.is_running_at(now + Duration::from_millis(400)));
}

#[test]
fn a_spring_channel_matches_the_stepped_spring() {
    let now = Instant::now();
    let spring = Spring::bouncy(0.42, 0.14);
    let mut c = Channel::new(0.);
    c.animate_at(1., Curve::Spring(spring), 0., false, now);
    let mut stepped = Animated::new(spring, 0.);
    stepped.set_target(1.);
    for _ in 0..150 {
        stepped.step(0.001, false);
    }
    let closed = c.value_at(now + Duration::from_millis(150));
    assert!(
        (closed - stepped.value()).abs() < 0.01,
        "{closed} vs {}",
        stepped.value()
    );
}

#[test]
fn retargeting_a_channel_keeps_its_value() {
    let now = Instant::now();
    let mut c = Channel::new(0.);
    c.animate_at(1., Curve::Spring(Spring::bouncy(0.4, 0.)), 0., false, now);
    let mid = now + Duration::from_millis(100);
    let before = c.value_at(mid);
    c.animate_at(0., Curve::Spring(Spring::bouncy(0.4, 0.)), 0., false, mid);
    assert!((c.value_at(mid) - before).abs() < 1e-4);
}

#[test]
fn a_channel_under_reduce_motion_jumps() {
    let mut c = Channel::new(0.);
    c.animate(5., Curve::Ease(1., ease::OUT), 1., true);
    assert_eq!(c.value(), 5.);
}

#[test]
fn disclosure_opens_slower_than_it_closes() {
    assert_eq!(disclosure(true), Curve::Ease(0.22, ease::OUT));
    assert_eq!(disclosure(false), Curve::Ease(0.14, ease::OUT));
}

#[test]
fn keyframes_hold_then_ease() {
    let open = |t| keyframes(&[0., 0., 12.], &[0., 0.4, 1.], 0.6, ease::OUT, t);
    assert_eq!(open(0.), 0.);
    assert_eq!(open(0.2), 0.);
    assert!((open(0.6) - 12.).abs() < 1e-3);
    let close = |t| keyframes(&[12., 0., 12.], &[0., 0.5, 1.], 0.42, ease::OUT, t);
    assert!(close(0.21).abs() < 1e-3);
    assert!((close(0.42) - 12.).abs() < 1e-3);
}

/// A spring to a pixel-sized target settles and says so. Near 237 an f32 step of a 1ms slice once came out
/// below one unit of the value, so the spring sat 0.00055 short for ever and an open Modal drew every frame.
#[test]
fn a_spring_to_a_pixel_sized_target_settles_in_a_few_seconds() {
    for spring in [Spring::PANEL, Spring::PRESS, Spring::LAYOUT, Spring::SWAP] {
        for target in [1., 40., 237., 900., 2400.] {
            let mut a = Animated::new(spring, 0.);
            a.set_target(target);
            let frames = (0..600).take_while(|_| a.step(1. / 60., false)).count();
            assert!(
                frames < 600,
                "{spring:?} to {target} still moves after 10 s at {}",
                a.value()
            );
            assert_eq!(a.value(), target, "a settled spring lands on its target");
        }
    }
}
