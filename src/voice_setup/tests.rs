use super::*;

#[test]
fn only_a_download_and_a_finished_setup_have_a_number() {
    assert_eq!(fraction(SetupPhase::Download(0.4)), Some(0.4));
    assert_eq!(fraction(SetupPhase::Ready), Some(1.));
    assert_eq!(fraction(SetupPhase::Prepare), None);
}

#[test]
fn a_bad_number_never_leaves_the_bar() {
    assert_eq!(fraction(SetupPhase::Download(7.)), Some(1.));
    assert_eq!(fraction(SetupPhase::Download(-1.)), Some(0.));
    assert_eq!(fraction(SetupPhase::Download(f32::NAN)), Some(0.));
}

#[test]
fn the_words_are_short_and_the_size_says_how_much_has_come_down() {
    assert_eq!(copy(SetupPhase::Download(0.42), 164.), ("Downloading speech model", Some("69 / 164 MB".to_string())));
    assert_eq!(copy(SetupPhase::Prepare, 164.), ("Getting ready", None));
    assert_eq!(copy(SetupPhase::Ready, 164.), ("Ready", None));
}
