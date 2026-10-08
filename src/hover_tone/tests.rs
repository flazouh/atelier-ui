use super::*;

#[test]
fn it_is_at_rest_until_the_pointer_or_a_picker_holds_it() {
    let mut tone = HoverTone::new();
    tone.sync(false, true);
    assert_eq!(tone.level(), 0.);
    tone.set_hovered(true);
    tone.sync(false, true);
    assert_eq!(tone.level(), 1.);
    tone.set_hovered(false);
    tone.sync(true, true);
    assert_eq!(
        tone.level(),
        1.,
        "an open picker holds it with the pointer gone"
    );
    tone.sync(false, true);
    assert_eq!(tone.level(), 0.);
}

#[test]
fn it_eases_and_asks_for_frames_until_it_settles() {
    let mut tone = HoverTone::new();
    tone.set_hovered(true);
    tone.sync(false, false);
    assert!(tone.is_moving(), "the tone is on its way, not there");
    assert!(tone.level() < 1.);
}
