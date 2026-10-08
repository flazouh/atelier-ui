use super::*;

#[test]
fn a_message_starts_clear_and_eight_pixels_low() {
    let f = frame(0., false);
    assert_eq!(f.opacity, 0.);
    assert_eq!(f.y, RISE);
    assert!(!f.settled);
}

#[test]
fn it_is_at_rest_well_inside_half_a_second() {
    let f = frame(0.4, false);
    assert!(f.settled);
    assert_eq!((f.opacity, f.y), (1., 0.));
}

#[test]
fn it_rises_on_beuis_spring() {
    let mid = frame(0.05, false);
    assert!(mid.opacity > 0.4 && mid.opacity < 1., "halfway in: {mid:?}");
    assert!(mid.y > 0. && mid.y < RISE);
    let ys: Vec<f32> = (1..40).map(|i| frame(i as f32 * 0.01, false).y).collect();
    assert!(
        ys.iter().all(|&y| y > -0.1),
        "a damping of 0.93 goes past its rest by a hair at most"
    );
}

#[test]
fn reduce_motion_shows_it_at_once() {
    assert_eq!(
        frame(0., true),
        PopFrame {
            opacity: 1.,
            y: 0.,
            settled: true
        }
    );
}

#[test]
fn the_spring_solves_every_damping() {
    for spring in [
        Spring::MESSAGE_POP,
        Spring::critical(30.),
        Spring {
            stiffness: 100.,
            damping: 60.,
            mass: 1.,
        },
    ] {
        assert_eq!(spring.position(0.), 0.);
        assert!(spring.position(0.01) > 0., "{spring:?} sets off");
        assert!(
            (spring.position(10.) - 1.).abs() < 1e-3,
            "{spring:?} comes to rest"
        );
    }
}
