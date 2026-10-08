use gpui_kit::SharedString;

use super::{Change, Field, Picker, Step, apply, shared_labels, shared_priority, shared_status};
use crate::{
    agent_look::AgentLook,
    task_model::{Activity, Assignee, Label, Priority, TaskData, TaskStatus},
    theme::Theme,
};

fn ids(list: &[&str]) -> Vec<SharedString> {
    list.iter()
        .map(|s| SharedString::from(s.to_string()))
        .collect()
}

fn tasks() -> Vec<TaskData> {
    let mut a = TaskData::new("a", "LAT-1", "A", TaskStatus::Todo);
    a.labels = vec![Label::new("bug", 1), Label::new("ui", 2)];
    let mut b = TaskData::new("b", "LAT-2", "B", TaskStatus::Todo);
    b.labels = vec![Label::new("bug", 1)];
    let c = TaskData::new("c", "LAT-3", "C", TaskStatus::Done);
    vec![a, b, c]
}

#[test]
fn the_status_picker_lists_every_status_and_starts_on_the_one_in_force() {
    let picker = Picker::status(Some(TaskStatus::InProgress));
    assert_eq!(picker.field(), Field::Status);
    let words: Vec<_> = picker
        .candidates()
        .iter()
        .map(|c| c.words.to_string())
        .collect();
    assert_eq!(
        words,
        [
            "Backlog",
            "Todo",
            "In Progress",
            "In Review",
            "Done",
            "Canceled"
        ]
    );
    assert_eq!(picker.cursor(), 2);
    assert_eq!(
        picker.choose(),
        Some(Change::Status(TaskStatus::InProgress))
    );
    assert_eq!(
        Picker::status(None).cursor(),
        0,
        "tasks that differ start on the first"
    );
}

#[test]
fn the_priority_picker_lists_urgent_first_and_no_priority_last() {
    let picker = Picker::priority(Some(Priority::Low));
    let words: Vec<_> = picker
        .candidates()
        .iter()
        .map(|c| c.words.to_string())
        .collect();
    assert_eq!(words, ["Urgent", "High", "Medium", "Low", "No priority"]);
    assert_eq!(picker.cursor(), 3);
}

#[test]
fn the_assignee_picker_offers_unassigned_first_then_people_and_agents() {
    let people = [
        Assignee::Person { name: "Ada".into() },
        Assignee::agent("Claude", AgentLook::neutral(&Theme::dark())),
    ];
    let picker = Picker::assignee(&people, Some(&"Claude".into()));
    let words: Vec<_> = picker
        .candidates()
        .iter()
        .map(|c| c.words.to_string())
        .collect();
    assert_eq!(words, ["Unassigned", "Ada", "Claude"]);
    assert_eq!(picker.cursor(), 2);
    assert_eq!(Picker::assignee(&people, None).cursor(), 0);
    assert_eq!(
        Picker::assignee(&people, None).choose(),
        Some(Change::Assignee(None))
    );
}

#[test]
fn typing_filters_the_candidates_and_the_cursor_goes_to_the_first_match() {
    let mut picker = Picker::status(Some(TaskStatus::Done));
    picker.type_text("PRO");
    assert_eq!(
        picker
            .shown()
            .iter()
            .map(|&i| picker.candidates()[i].words.to_string())
            .collect::<Vec<_>>(),
        ["In Progress"]
    );
    assert_eq!((picker.cursor(), picker.query()), (0, "PRO"));
    assert_eq!(
        picker.choose(),
        Some(Change::Status(TaskStatus::InProgress))
    );
    picker.type_text("zzz");
    assert!(picker.shown().is_empty());
    assert_eq!(
        picker.choose(),
        None,
        "nothing matches, so Enter does nothing"
    );
    for _ in 0..6 {
        picker.backspace();
    }
    assert_eq!(picker.shown().len(), 6);
}

#[test]
fn the_cursor_steps_within_the_matches_and_stops_at_the_ends() {
    let mut picker = Picker::status(None);
    picker.step(Step::Up);
    assert_eq!(picker.cursor(), 0);
    for _ in 0..10 {
        picker.step(Step::Down);
    }
    assert_eq!(picker.cursor(), 5);
    assert_eq!(picker.choose(), Some(Change::Status(TaskStatus::Canceled)));
    picker.type_text("zzz");
    picker.step(Step::Down);
    assert_eq!(picker.cursor(), 0);
}

#[test]
fn only_the_label_picker_stays_open_and_it_marks_what_it_applied() {
    assert!(!Picker::status(None).stays_open() && !Picker::priority(None).stays_open());
    let all = [Label::new("bug", 1), Label::new("ui", 2)];
    let mut picker = Picker::labels(&all, &[Label::new("bug", 1)]);
    assert!(picker.stays_open());
    assert_eq!(
        picker
            .candidates()
            .iter()
            .map(|c| c.chosen)
            .collect::<Vec<_>>(),
        [true, false]
    );
    picker.mark(&Label::new("ui", 2), true);
    assert_eq!(
        picker
            .candidates()
            .iter()
            .map(|c| c.chosen)
            .collect::<Vec<_>>(),
        [true, true]
    );
}

#[test]
fn changing_a_status_sets_it_notes_the_change_in_the_activity_and_bumps_the_time() {
    let mut list = tasks();
    let n = apply(
        &mut list,
        &ids(&["a", "c"]),
        &Change::Status(TaskStatus::InProgress),
        "Ada",
        500,
    );
    assert_eq!(n, 2);
    assert_eq!(
        (list[0].status, list[0].updated_at),
        (TaskStatus::InProgress, 500)
    );
    assert_eq!(
        list[0].activity,
        vec![Activity::StatusChanged {
            by: "Ada".into(),
            from: TaskStatus::Todo,
            to: TaskStatus::InProgress,
            at: 500
        }]
    );
    assert_eq!(list[1].status, TaskStatus::Todo, "b was not named");
    assert_eq!(list[1].updated_at, 0);
}

#[test]
fn a_change_that_changes_nothing_touches_nothing() {
    let mut list = tasks();
    assert_eq!(
        apply(
            &mut list,
            &ids(&["a"]),
            &Change::Status(TaskStatus::Todo),
            "Ada",
            9
        ),
        0
    );
    assert_eq!(
        apply(
            &mut list,
            &ids(&["a"]),
            &Change::Priority(Priority::None),
            "Ada",
            9
        ),
        0
    );
    assert_eq!(
        apply(&mut list, &ids(&["a"]), &Change::Assignee(None), "Ada", 9),
        0
    );
    assert_eq!(
        apply(
            &mut list,
            &ids(&["zzz"]),
            &Change::Status(TaskStatus::Done),
            "Ada",
            9
        ),
        0
    );
    assert!(
        list.iter()
            .all(|t| t.updated_at == 0 && t.activity.is_empty())
    );
}

#[test]
fn priority_and_assignee_apply_to_every_named_task() {
    let mut list = tasks();
    assert_eq!(
        apply(
            &mut list,
            &ids(&["a", "b", "c"]),
            &Change::Priority(Priority::Urgent),
            "Ada",
            7
        ),
        3
    );
    assert!(
        list.iter()
            .all(|t| t.priority == Priority::Urgent && t.updated_at == 7)
    );
    let claude = Assignee::agent("Claude", AgentLook::neutral(&Theme::dark()));
    assert_eq!(
        apply(
            &mut list,
            &ids(&["b"]),
            &Change::Assignee(Some(claude)),
            "Ada",
            8
        ),
        1
    );
    assert!(list[1].assignee.as_ref().is_some_and(Assignee::is_agent));
    assert_eq!(
        apply(&mut list, &ids(&["b"]), &Change::Assignee(None), "Ada", 9),
        1
    );
    assert!(list[1].assignee.is_none());
}

#[test]
fn a_label_goes_on_the_tasks_that_lack_it_and_comes_off_when_all_have_it() {
    let mut list = tasks();
    let bug = Label::new("bug", 1);
    let ui = Label::new("ui", 2);
    // "ui" is on a only: it goes on b and c.
    assert_eq!(
        apply(
            &mut list,
            &ids(&["a", "b", "c"]),
            &Change::ToggleLabel(ui.clone()),
            "Ada",
            5
        ),
        2
    );
    assert!(list.iter().all(|t| t.labels.contains(&ui)));
    // Now all have "ui": the same choice takes it off all.
    assert_eq!(
        apply(
            &mut list,
            &ids(&["a", "b", "c"]),
            &Change::ToggleLabel(ui.clone()),
            "Ada",
            6
        ),
        3
    );
    assert!(list.iter().all(|t| !t.labels.contains(&ui)));
    // "bug" is on a and b only: c gets it, a and b stay as they were.
    assert_eq!(
        apply(
            &mut list,
            &ids(&["a", "b", "c"]),
            &Change::ToggleLabel(bug.clone()),
            "Ada",
            7
        ),
        1
    );
    assert_eq!(
        list[0].updated_at, 6,
        "a was not touched by the last change"
    );
    assert!(list[2].labels.contains(&bug));
}

#[test]
fn what_the_tasks_share_is_what_a_picker_starts_on() {
    let list = tasks();
    let refs = |ids: &[usize]| ids.iter().map(|&i| &list[i]).collect::<Vec<_>>();
    assert_eq!(shared_status(&refs(&[0, 1])), Some(TaskStatus::Todo));
    assert_eq!(shared_status(&refs(&[0, 2])), None);
    assert_eq!(shared_status(&[]), None);
    assert_eq!(shared_priority(&refs(&[0, 1])), Some(Priority::None));
    assert_eq!(shared_labels(&refs(&[0, 1])), vec![Label::new("bug", 1)]);
    assert!(shared_labels(&refs(&[0, 2])).is_empty());
    assert!(shared_labels(&[]).is_empty());
}

#[test]
fn a_shift_of_status_moves_each_task_from_where_it_is_and_leaves_the_ends() {
    use super::shift_status;
    let tasks = vec![
        TaskData::new("a", "LAT-1", "A", TaskStatus::Todo),
        TaskData::new("b", "LAT-2", "B", TaskStatus::Done),
        TaskData::new("c", "LAT-3", "C", TaskStatus::Canceled),
    ];
    let ids: Vec<gpui_kit::SharedString> = vec!["a".into(), "b".into(), "c".into(), "zzz".into()];
    assert_eq!(
        shift_status(&tasks, &ids, true),
        vec![
            ("a".into(), Change::Status(TaskStatus::InProgress)),
            ("b".into(), Change::Status(TaskStatus::Canceled))
        ]
    );
    assert_eq!(
        shift_status(&tasks, &ids, false),
        vec![
            ("a".into(), Change::Status(TaskStatus::Backlog)),
            ("b".into(), Change::Status(TaskStatus::InReview)),
            ("c".into(), Change::Status(TaskStatus::Done))
        ]
    );
}
