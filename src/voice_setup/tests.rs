use super::*;

#[test]
fn only_a_download_and_a_finished_setup_have_a_number() {
    assert_eq!(fraction(SetupPhase::Download(0.4)), Some(0.4));
    assert_eq!(fraction(SetupPhase::Ready), Some(1.));
    assert_eq!(fraction(SetupPhase::Prepare), None);
}

#[test]
fn a_bad_number_never_leaves_the_track() {
    assert_eq!(fraction(SetupPhase::Download(7.)), Some(1.));
    assert_eq!(fraction(SetupPhase::Download(-1.)), Some(0.));
    assert_eq!(fraction(SetupPhase::Download(f32::NAN)), Some(0.));
}

#[test]
fn the_words_say_how_much_has_come_down() {
    let (words, percent) = copy(SetupPhase::Download(0.42), 164.);
    assert_eq!(words, "Downloading speech model · 69 of 164 MB");
    assert_eq!(percent.as_deref(), Some("42%"));
    assert_eq!(copy(SetupPhase::Prepare, 164.).1, None);
    assert_eq!(copy(SetupPhase::Ready, 164.).0, "Ready");
}

#[test]
fn the_fill_eases_toward_its_number_without_passing_it() {
    let one = pour(0., 1., 0.016);
    let two = pour(one, 1., 0.016);
    assert!(one > 0. && two > one && two < 1.);
    assert!(pour(1., 0.5, 0.016) < 1. && pour(1., 0.5, 0.016) > 0.5);
}

#[test]
fn the_sweep_stays_on_the_track_and_turns_around() {
    for k in 0..400 {
        let left = sweep_left(k as f32 * 0.02);
        assert!((0. ..=1. - SWEEP_WIDTH + 1e-6).contains(&left), "{left}");
    }
    assert!(sweep_left(0.) < 1e-6);
    assert!((sweep_left(SWEEP_SECONDS) - (1. - SWEEP_WIDTH)).abs() < 1e-6);
    assert!(sweep_left(2. * SWEEP_SECONDS) < 1e-6);
}

#[test]
fn the_light_enters_from_the_left_and_leaves_on_the_right() {
    assert!(sheen_left(0.) < 0.);
    assert!(sheen_left(SHEEN_SECONDS * 0.99) > 0.9);
    assert!(sheen_left(SHEEN_SECONDS) < 0.);
}
