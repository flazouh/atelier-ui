use super::*;

fn row() -> SubagentRow {
    SubagentRow::new(
        "explore",
        AgentLook::neutral(&crate::theme::Theme::dark()),
        "Explore",
        "Find the diff parser",
    )
}

#[test]
fn a_running_row_shows_its_name_and_task() {
    let row = row().elapsed("12s");
    assert_eq!(row.title(), "Explore");
    assert_eq!(row.detail(), "Find the diff parser");
    assert_eq!(row.status_label(), "12s");
}

#[test]
fn the_live_tool_call_takes_the_tasks_place_until_it_finishes() {
    let running = row().tool("Read theme.rs");
    assert_eq!(running.detail(), "Read theme.rs");
    assert_eq!(running.finished(Some(3)).detail(), "Find the diff parser");
}

#[test]
fn a_finished_row_says_done() {
    assert_eq!(row().finished(Some(38)).status_label(), "Done in 38s");
    assert_eq!(row().finished(None).status_label(), "Done");
}

#[test]
fn tool_calls_count_in_words() {
    assert_eq!(tool_calls_text(1), "1 tool call");
    assert_eq!(tool_calls_text(12), "12 tool calls");
    assert_eq!(tool_calls_text(0), "0 tool calls");
}
