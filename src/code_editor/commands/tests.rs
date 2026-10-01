// A selection list with one range is the common case here, not a typo for the range itself.
#![allow(clippy::single_range_in_vec_init)]

use super::*;

/// Text with `|` for each caret and `[` `]` around each selection, split into the text and ranges.
fn marked(source: &str) -> (String, Vec<Range<usize>>) {
    let mut text = String::new();
    let mut selections = Vec::new();
    let mut open = None;
    for c in source.chars() {
        match c {
            '|' => selections.push(text.len()..text.len()),
            '[' => open = Some(text.len()),
            ']' => selections.push(open.take().expect("a [ before ]")..text.len()),
            c => text.push(c),
        }
    }
    (text, selections)
}

/// The text after `edit`, with its selections marked the same way.
fn applied(text: &str, edit: &Edit) -> String {
    let mut out = String::new();
    out.push_str(&text[..edit.range.start]);
    out.push_str(&edit.text);
    out.push_str(&text[edit.range.end..]);
    let mut selections = edit.selections.clone();
    selections.sort_by_key(|s| std::cmp::Reverse(s.start));
    for s in selections {
        if s.is_empty() {
            out.insert(s.start, '|');
        } else {
            out.insert(s.end, ']');
            out.insert(s.start, '[');
        }
    }
    out
}

fn run(source: &str, command: impl Fn(&str, &[Range<usize>]) -> Option<Edit>) -> String {
    let (text, selections) = marked(source);
    applied(&text, &command(&text, &selections).expect("the command applies"))
}

fn comment(text: &str, selections: &[Range<usize>]) -> Option<Edit> {
    toggle_comment(text, selections, "//")
}

#[test]
fn a_language_without_line_comments_gets_no_marker() {
    assert_eq!(comment_prefix("rust"), Some("//"));
    assert_eq!(comment_prefix("python"), Some("#"));
    assert_eq!(comment_prefix("json"), None);
    assert_eq!(comment_prefix("markdown"), None);
}

#[test]
fn a_selection_that_ends_at_a_line_start_leaves_that_line_out() {
    let text = "a\nbb\ncc\nd";
    assert_eq!(line_span(text, &(2..8)), 2..7, "lines 2 and 3, not 4");
    assert_eq!(line_span(text, &(3..3)), 2..4, "a caret takes its own line");
    assert_eq!(line_span(text, &(9..9)), 8..9, "the last line has no newline");
}

#[test]
fn home_goes_to_the_indent_then_to_column_zero_for_every_caret() {
    let (text, carets) = marked("fn a() {\n    let x| = 1;\n    |y\n}");
    assert_eq!(smart_home(&text, &carets), vec![13..13, 24..24], "one goes to its indent, one already there to column zero");
    assert_eq!(smart_home(&text, &[13..13]), vec![9..9], "second press: column zero");
    assert_eq!(smart_home(&text, &[9..9]), vec![13..13], "third press: back to the indent");
}

#[test]
fn a_comment_goes_in_at_the_caret_line_and_the_caret_keeps_its_place() {
    assert_eq!(run("fn a() {\n    le|t x = 1;\n}", comment), "fn a() {\n    // le|t x = 1;\n}");
}

#[test]
fn a_block_is_commented_at_its_shallowest_indent_and_blank_lines_are_skipped() {
    assert_eq!(run("[    if a {\n\n        b();\n    }]", comment), "[    // if a {\n\n    //     b();\n    // }]");
}

#[test]
fn a_block_that_is_all_comments_is_uncommented() {
    assert_eq!(run("[    // if a {\n    //     b();\n    // }]", comment), "[    if a {\n        b();\n    }]");
}

#[test]
fn a_mix_of_comments_and_code_is_commented_not_uncommented() {
    assert_eq!(run("[// a\nb]", comment), "[// // a\n// b]");
}

#[test]
fn a_caret_inside_a_removed_marker_lands_where_the_marker_was() {
    assert_eq!(run("  /|/ x", comment), "  |x");
}

#[test]
fn hash_comments_work_the_same_way() {
    assert_eq!(run("a = |1", |t, s| toggle_comment(t, s, "#")), "# a = |1");
    assert_eq!(run("# a = |1", |t, s| toggle_comment(t, s, "#")), "a = |1");
}

#[test]
fn only_blank_lines_have_nothing_to_comment() {
    let (text, carets) = marked("a\n|\nb");
    assert_eq!(toggle_comment(&text, &carets, "//"), None);
}

#[test]
fn every_cursor_comments_its_own_line_in_one_edit() {
    assert_eq!(run("a|\nb\nc|", comment), "// a|\nb\n// c|");
    assert_eq!(run("// a|\nb\n// c|", comment), "a|\nb\nc|", "all commented, so all uncommented");
}

#[test]
fn two_cursors_on_one_line_comment_it_once() {
    assert_eq!(run("a|b|c", comment), "// a|b|c");
}

#[test]
fn a_line_moves_up_past_its_neighbour_and_the_caret_rides_along() {
    assert_eq!(run("one\ntw|o\nthree", |t, s| move_lines(t, s, true)), "tw|o\none\nthree");
}

#[test]
fn a_block_moves_down_past_its_neighbour() {
    assert_eq!(run("[one\ntwo]\nthree", |t, s| move_lines(t, s, false)), "three\n[one\ntwo]");
}

#[test]
fn separate_cursors_each_move_their_own_line() {
    assert_eq!(run("a\nb|\nc\nd|", |t, s| move_lines(t, s, true)), "b|\na\nd|\nc");
}

#[test]
fn nothing_moves_past_the_top_or_the_bottom() {
    let (text, carets) = marked("o|ne\ntwo");
    assert_eq!(move_lines(&text, &carets, true), None);
    let (text, carets) = marked("one\ntw|o");
    assert_eq!(move_lines(&text, &carets, false), None);
    let (text, carets) = marked("o|ne\ntwo\nth|ree");
    assert_eq!(move_lines(&text, &carets, true), None, "one block at the top holds every block");
}

#[test]
fn a_duplicate_goes_below_and_takes_the_caret() {
    assert_eq!(run("a\nb|b\nc", duplicate_lines), "a\nbb\nb|b\nc");
    assert_eq!(run("[a\nb]", duplicate_lines), "a\nb\n[a\nb]");
    assert_eq!(run("a|\nb\nc|", duplicate_lines), "a\na|\nb\nc\nc|", "each cursor duplicates its line");
}

#[test]
fn a_deleted_line_hands_its_column_to_the_next_line() {
    assert_eq!(run("one\ntw|o\nthree", delete_lines), "one\nth|ree");
    assert_eq!(run("one\ntwo|two\nx", delete_lines), "one\nx|", "a short next line clamps the column");
}

#[test]
fn deleting_the_last_line_takes_the_newline_before_it() {
    assert_eq!(run("one\ntw|o", delete_lines), "on|e");
    assert_eq!(run("on|ly", delete_lines), "|");
}

#[test]
fn separate_cursors_each_delete_their_own_line() {
    assert_eq!(run("a|\nb\nc|\nd", delete_lines), "b|\nd|", "each keeps its column on the line that slid up");
}

#[test]
fn copy_with_nothing_selected_takes_every_caret_line_once() {
    let (text, carets) = marked("one\ntw|o\nthree");
    assert_eq!(whole_lines(&text, &carets), (vec![4..8], "two\n".to_string()));
    let (text, carets) = marked("one\ntw|o");
    assert_eq!(whole_lines(&text, &carets), (vec![4..7], "two\n".to_string()), "the last line copies as a line");
    let (text, carets) = marked("o|n|e\ntwo\nthr|ee");
    assert_eq!(whole_lines(&text, &carets), (vec![0..4, 8..13], "one\nthree\n".to_string()));
}

#[test]
fn a_whole_line_pastes_above_the_caret_line() {
    assert_eq!(run("one\ntw|o", |t, s| whole_line_paste(t, s, "new\n")), "one\nnew\ntw|o");
    assert_eq!(
        run("o|n|e\ntw|o", |t, s| whole_line_paste(t, s, "new\n")),
        "new\no|n|e\nnew\ntw|o",
        "once above each caret line"
    );
}

#[test]
fn select_all_then_delete_lines_empties_the_buffer() {
    assert_eq!(run("[a\nb\n]", delete_lines), "|");
}
