use super::*;

#[test]
fn a_running_card_counts_its_tool_calls() {
    assert_eq!(status_line(None, 12), ["12 tool calls"]);
    assert_eq!(status_line(None, 1), ["1 tool call"]);
}

#[test]
fn a_finished_card_says_how_long_it_ran_and_how_many_calls_it_made() {
    assert_eq!(status_line(Some(Some(38)), 12), ["Done in 38s", "12 tool calls"]);
    assert_eq!(status_line(Some(None), 1), ["Done", "1 tool call"]);
}

#[test]
fn no_part_of_the_card_parts_its_text_with_a_middle_dot() {
    for finished in [None, Some(None), Some(Some(38))] {
        for segment in status_line(finished, 12) {
            assert!(!segment.contains('\u{b7}'), "{segment:?}");
        }
    }
}
