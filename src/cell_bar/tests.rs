use super::*;
use gpui_kit::hsla;

#[test]
fn cells_fill_from_the_left_in_step_with_the_number() {
    assert_eq!(whole_cells(CELLS, 0.), 0);
    assert_eq!(whole_cells(CELLS, 0.5), CELLS / 2);
    assert_eq!(whole_cells(CELLS, 1.), CELLS);
    assert_eq!(whole_cells(30, 0.58), 17);
}

#[test]
fn a_bad_number_never_leaves_the_bar() {
    assert_eq!(whole_cells(CELLS, 7.), CELLS);
    assert_eq!(whole_cells(CELLS, -1.), 0);
    assert_eq!(whole_cells(CELLS, f32::NAN), 0);
}

#[test]
fn the_head_cell_is_part_lit_between_two_whole_cells() {
    let fill = 3.5 / CELLS as f32;
    assert_eq!(lit(2, CELLS, fill), 1.);
    assert!((lit(3, CELLS, fill) - 0.5).abs() < 1e-4);
    assert_eq!(lit(4, CELLS, fill), 0.);
}

#[test]
fn the_fill_eases_toward_its_number_without_passing_it() {
    let one = pour(0., 1., 0.016);
    let two = pour(one, 1., 0.016);
    assert!(one > 0. && two > one && two < 1.);
    assert!(pour(1., 0.5, 0.016) < 1. && pour(1., 0.5, 0.016) > 0.5);
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
fn a_dark_cell_is_the_wash_and_a_lit_one_is_the_bars_color() {
    let color = hsla(0.1, 0.9, 0.4, 1.);
    let dark = hsla(0., 0., 0.1, DARK_CELL);
    assert_eq!(cell_color(color, dark, 0.), dark);
    assert_eq!(cell_color(color, dark, 1.), color);
    let half = cell_color(color, dark, 0.5);
    assert!(half.a > dark.a && half.a < 1. && half.h == color.h);
}
