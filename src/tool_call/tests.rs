use super::*;

#[test]
fn a_run_that_starts_running_opens() {
    assert!(should_open(ToolStatus::Running, false, false, false));
}

#[test]
fn output_arriving_on_an_already_running_call_opens_it() {
    // The bug: a Running call with no output yet, then output shows up while it is still running.
    assert!(should_open(ToolStatus::Running, true, false, true));
}

#[test]
fn output_that_was_already_there_does_not_reopen_it() {
    assert!(!should_open(ToolStatus::Running, true, true, true));
}

#[test]
fn a_call_with_no_output_at_all_never_opens() {
    assert!(!should_open(ToolStatus::Running, true, false, false));
}

#[test]
fn a_finished_call_does_not_open_just_because_it_has_a_body() {
    assert!(!should_open(ToolStatus::Done, false, false, true));
}
