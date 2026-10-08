use super::{
    consts::{SPARE, ZOOM},
    drift, reveal, zoom,
};
use super::helpers::fade;
use crate::theme::Theme;

#[test]
fn a_piece_waits_for_its_turn_then_comes_in_whole() {
    assert!(reveal(0., 0.5, 0.7) < 1e-6);
    assert!(reveal(0.5, 0.5, 0.7) < 1e-6);
    assert_eq!(reveal(5., 0.5, 0.7), 1.);
    let half = reveal(0.85, 0.5, 0.7);
    assert!(half > 0.5 && half < 1., "ease out has come over half its way by half its time: {half}");
}

#[test]
fn the_picture_starts_out_and_ends_in_its_place() {
    assert!((zoom(0.) - ZOOM).abs() < 1e-4);
    assert!(zoom(10.) < 1e-6);
    assert!(zoom(0.8) < zoom(0.4), "it only moves one way");
}

#[test]
fn the_drift_is_asleep_at_first_and_never_shows_a_border() {
    let (x, y) = drift(0.);
    assert!(x.abs() < 1e-6 && y.abs() < 1e-6, "asleep at the start: {x}, {y}");
    for step in 0..2_000 {
        let (x, y) = drift(step as f32 * 0.1);
        assert!(x.abs() < SPARE && y.abs() < SPARE, "the drift at {step}: {x}, {y}");
    }
    assert!((0..200).any(|s| drift(3. + s as f32 * 0.1).0.abs() > 5.), "it does move");
}

#[test]
fn a_fade_keeps_the_colour_and_scales_only_the_strength() {
    let colour = Theme::dark().foreground;
    let half = fade(colour, 0.5);
    assert_eq!((half.h, half.s, half.l), (colour.h, colour.s, colour.l));
    assert!((half.a - colour.a * 0.5).abs() < 1e-6);
}
