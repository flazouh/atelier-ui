use gpui_kit::{Hsla, HighlightStyle};

use super::{gutter_numbers, whole_text_runs};

fn style() -> HighlightStyle {
    HighlightStyle { color: Some(Hsla::default()), ..Default::default() }
}

#[test]
fn the_gutter_is_one_number_per_line() {
    assert_eq!(gutter_numbers(3).as_ref(), "1\n2\n3");
    assert_eq!(gutter_numbers(1).as_ref(), "1");
}

#[test]
fn line_runs_move_to_their_place_in_the_whole_text() {
    let lines = vec![vec![(0..2, style())], vec![], vec![(1..3, style())]];
    let runs = whole_text_runs("ab\n\nxyz", &lines);
    let ranges: Vec<_> = runs.into_iter().map(|(r, _)| r).collect();
    assert_eq!(ranges, vec![0..2, 5..7]);
}

#[test]
fn fewer_lines_of_runs_than_lines_of_text_is_fine() {
    let lines = vec![vec![(0..1, style())]];
    assert_eq!(whole_text_runs("a\nb\nc", &lines).len(), 1);
}
