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

#[test]
fn the_fill_eases_toward_its_number_without_passing_it() {
    let one = pour(0., 1., 0.016);
    let two = pour(one, 1., 0.016);
    assert!(one > 0. && two > one && two < 1.);
    assert!(pour(1., 0.5, 0.016) < 1. && pour(1., 0.5, 0.016) > 0.5);
}

#[test]
fn cells_fill_from_the_left_in_step_with_the_number() {
    let on = |fill: f32| (0..CELLS).filter(|&i| lit(i, CELLS, fill) >= 1.).count();
    assert_eq!(on(0.), 0);
    assert_eq!(on(0.5), CELLS / 2);
    assert_eq!(on(1.), CELLS);
}

#[test]
fn the_head_cell_is_part_lit_between_two_whole_cells() {
    // 3.5 cells of 14.
    let fill = 3.5 / CELLS as f32;
    assert_eq!(lit(2, CELLS, fill), 1.);
    assert!((lit(3, CELLS, fill) - 0.5).abs() < 1e-4);
    assert_eq!(lit(4, CELLS, fill), 0.);
}

#[test]
fn the_loading_cell_walks_the_row_and_starts_again() {
    let at = |s: f32| loading_cell(s, CELLS, true);
    assert_eq!(at(0.), 0);
    assert_eq!(at(1. / STEPS_PER_SECOND), 1);
    assert!((0..200).all(|k| at(k as f32 * 0.03) < CELLS));
    assert_eq!(at(CELLS as f32 / STEPS_PER_SECOND), 0);
}

#[test]
fn the_loading_cell_rests_in_the_middle_without_motion() {
    assert_eq!(loading_cell(0., CELLS, false), CELLS / 2);
    assert_eq!(loading_cell(9., CELLS, false), CELLS / 2);
}

#[test]
fn a_dark_cell_is_faint_and_a_lit_one_is_solid() {
    let base = crate::voice_waves::amber();
    assert!(cell_color(base, 0.).a < 0.2);
    assert!(cell_color(base, 1.).a > 0.99);
    assert!(cell_color(base, 0.5).a > cell_color(base, 0.).a);
}
