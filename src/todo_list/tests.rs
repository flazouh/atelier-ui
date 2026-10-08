use super::*;

#[test]
fn progress_counts_only_done_steps() {
    let todos = vec![
        Todo::new("read", "Read the code", TodoStatus::Done),
        Todo::new("fix", "Fix the bug", TodoStatus::InProgress),
        Todo::new("test", "Run the tests", TodoStatus::Pending),
    ];
    assert_eq!(progress(&todos), (1, 3));
}

#[test]
fn an_empty_plan_has_no_progress() {
    assert_eq!(progress(&[]), (0, 0));
}

#[test]
fn inserting_a_step_in_the_middle_only_adds_the_new_row() {
    let prev = vec![
        ("a".into(), TodoStatus::Done, None),
        ("b".into(), TodoStatus::Pending, None),
    ];
    let todos = vec![
        Todo::new("a", "A", TodoStatus::Done),
        Todo::new("c", "C", TodoStatus::Pending),
        Todo::new("b", "B", TodoStatus::Pending),
    ];
    assert_eq!(
        plan_rows(&prev, &todos),
        vec![RowPlan::Reuse(0), RowPlan::New, RowPlan::Reuse(1)]
    );
}

#[test]
fn removing_a_step_drops_only_its_own_row() {
    let prev = vec![
        ("a".into(), TodoStatus::Done, None),
        ("b".into(), TodoStatus::Pending, None),
        ("c".into(), TodoStatus::Pending, None),
    ];
    let todos = vec![
        Todo::new("a", "A", TodoStatus::Done),
        Todo::new("c", "C", TodoStatus::Pending),
    ];
    assert_eq!(
        plan_rows(&prev, &todos),
        vec![RowPlan::Reuse(0), RowPlan::Reuse(2)]
    );
}

#[test]
fn a_progress_change_retargets_instead_of_reusing() {
    let prev = vec![("a".into(), TodoStatus::InProgress, Some(20.))];
    let todos = vec![Todo::new("a", "A", TodoStatus::InProgress).progress(60.)];
    assert_eq!(plan_rows(&prev, &todos), vec![RowPlan::Retarget(0)]);
}

#[test]
fn an_unmatched_id_always_starts_a_new_row() {
    assert_eq!(
        plan_rows(&[], &[Todo::new("a", "A", TodoStatus::Pending)]),
        vec![RowPlan::New]
    );
}
