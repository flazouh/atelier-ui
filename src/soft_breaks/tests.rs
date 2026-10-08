use super::keep;

#[test]
fn the_lines_of_a_paragraph_keep_their_breaks() {
    assert_eq!(keep("one\ntwo\nthree"), "one  \ntwo  \nthree");
}

#[test]
fn a_blank_line_and_the_last_line_get_none() {
    assert_eq!(keep("one\ntwo\n\nthree\n"), "one  \ntwo\n\nthree\n");
}

#[test]
fn a_fenced_block_is_left_as_written() {
    let text = "intro\n```rust\nlet a = 1;\nlet b = 2;\n```\nafter";
    assert_eq!(keep(text), "intro\n```rust\nlet a = 1;\nlet b = 2;\n```\nafter");
    let long = "````\n```\nstill code\n````\ndone\nmore";
    assert_eq!(keep(long), "````\n```\nstill code\n````\ndone  \nmore");
}

#[test]
fn a_table_and_a_hard_break_are_left_alone() {
    assert_eq!(keep("| a | b |\n|---|---|\n| 1 | 2 |"), "| a | b |\n|---|---|\n| 1 | 2 |");
    assert_eq!(keep("already  \nbroken\\\nalso"), "already  \nbroken\\\nalso");
}

#[test]
fn list_lines_and_headings_take_a_break_harmlessly() {
    assert_eq!(keep("# Title\n- a\n- b"), "# Title  \n- a  \n- b");
}

#[test]
fn text_with_no_newline_is_unchanged() {
    assert_eq!(keep("just one line"), "just one line");
    assert_eq!(keep(""), "");
}
