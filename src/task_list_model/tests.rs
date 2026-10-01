use gpui_kit::SharedString;

use crate::task_edit::Change;
use super::{Cursor, Filters, Folds, Move, Row, Sort, SortKey, cycle, group, rows};
use crate::{
    agent_look::AgentLook,
    task_model::{Assignee, Label, Priority, TaskData, TaskStatus},
    theme::Theme,
};

fn task(n: u32, status: TaskStatus, priority: Priority, who: Option<&str>, labels: &[&str], updated: u64) -> TaskData {
    let mut t = TaskData::new(format!("t{n}"), format!("LAT-{n}"), format!("Task number {n}"), status);
    t.priority = priority;
    t.assignee = who.map(|w| {
        if w == "Claude" {
            Assignee::agent(w, AgentLook::neutral(&Theme::dark()))
        } else {
            Assignee::Person { name: w.into() }
        }
    });
    t.labels = labels.iter().map(|l| Label::new(*l, 1)).collect();
    t.updated_at = updated;
    t.created_at = updated / 2;
    t
}

fn sample() -> Vec<TaskData> {
    use Priority::{High, Low, Medium, Urgent};
    use TaskStatus::*;
    vec![
        task(1, Todo, High, Some("ada"), &["bug"], 50),
        task(2, InProgress, Urgent, Some("Claude"), &["bug", "ui"], 90),
        task(3, Todo, Low, None, &[], 10),
        task(4, Done, Medium, Some("ada"), &["ui"], 70),
        task(5, InProgress, Priority::None, Some("grace"), &[], 30),
        task(6, Backlog, High, Some("ada"), &["ui"], 20),
        task(7, Canceled, Low, None, &["bug"], 5),
        task(8, InProgress, High, Some("ada"), &[], 95),
    ]
}

fn keys(tasks: &[TaskData], indices: &[usize]) -> Vec<String> {
    indices.iter().map(|&i| tasks[i].key.to_string()).collect()
}

fn kept(filters: &Filters) -> Vec<String> {
    let tasks = sample();
    let groups = group(&tasks, filters, "ada", Sort::default(), &TaskStatus::LIST_ORDER, false);
    let mut all: Vec<usize> = groups.iter().flat_map(|g| g.tasks.clone()).collect();
    all.sort();
    keys(&tasks, &all)
}

#[test]
fn no_filter_keeps_every_task() {
    assert!(Filters::default().is_empty());
    assert_eq!(kept(&Filters::default()).len(), 8);
}

#[test]
fn each_quick_filter_keeps_what_it_names() {
    assert_eq!(kept(&Filters { mine: true, ..Filters::default() }), ["LAT-1", "LAT-4", "LAT-6", "LAT-8"]);
    assert_eq!(kept(&Filters { assignee: Some("Claude".into()), ..Filters::default() }), ["LAT-2"]);
    assert_eq!(kept(&Filters { label: Some("ui".into()), ..Filters::default() }), ["LAT-2", "LAT-4", "LAT-6"]);
    assert_eq!(kept(&Filters { priority: Some(Priority::High), ..Filters::default() }), ["LAT-1", "LAT-6", "LAT-8"]);
    assert_eq!(kept(&Filters { priority: Some(Priority::None), ..Filters::default() }), ["LAT-5"]);
}

#[test]
fn filters_combine_with_and() {
    let both = Filters { mine: true, label: Some("ui".into()), ..Filters::default() };
    assert_eq!(kept(&both), ["LAT-4", "LAT-6"]);
    let none = Filters { assignee: Some("grace".into()), label: Some("bug".into()), ..Filters::default() };
    assert!(kept(&none).is_empty());
}

#[test]
fn a_text_filter_looks_in_the_title_and_the_key_whatever_the_case() {
    assert_eq!(kept(&Filters { text: "NUMBER 3".into(), ..Filters::default() }), ["LAT-3"]);
    assert_eq!(kept(&Filters { text: "lat-7".into(), ..Filters::default() }), ["LAT-7"]);
    assert!(kept(&Filters { text: "nothing like this".into(), ..Filters::default() }).is_empty());
}

#[test]
fn a_task_nobody_holds_is_not_mine_and_not_any_assignee() {
    assert!(!kept(&Filters { mine: true, ..Filters::default() }).contains(&"LAT-3".to_string()));
    assert!(!kept(&Filters { assignee: Some("ada".into()), ..Filters::default() }).contains(&"LAT-3".to_string()));
}

#[test]
fn groups_follow_the_order_given_and_leave_out_empty_ones_in_a_list() {
    let tasks = sample();
    let groups = group(&tasks, &Filters::default(), "ada", Sort::default(), &TaskStatus::LIST_ORDER, false);
    let statuses: Vec<_> = groups.iter().map(|g| g.status.words()).collect();
    assert_eq!(statuses, ["In Progress", "Todo", "Backlog", "Done", "Canceled"], "nothing is In Review");
    let board = group(&tasks, &Filters::default(), "ada", Sort::default(), &TaskStatus::BOARD_ORDER, true);
    assert_eq!(board.len(), 6);
    assert!(board[3].tasks.is_empty() && board[3].status == TaskStatus::InReview, "a board keeps its empty column");
}

#[test]
fn a_filter_that_empties_a_group_removes_it_from_the_list() {
    let tasks = sample();
    let groups = group(&tasks, &Filters { label: Some("ui".into()), ..Filters::default() }, "ada", Sort::default(), &TaskStatus::LIST_ORDER, false);
    let statuses: Vec<_> = groups.iter().map(|g| g.status).collect();
    assert_eq!(statuses, [TaskStatus::InProgress, TaskStatus::Backlog, TaskStatus::Done]);
}

fn sorted_keys(key: SortKey, reversed: bool) -> Vec<String> {
    let tasks = sample();
    let groups = group(&tasks, &Filters::default(), "ada", Sort { key, reversed }, &[TaskStatus::InProgress], false);
    keys(&tasks, &groups[0].tasks)
}

#[test]
fn a_group_sorts_by_priority_then_by_the_latest_change() {
    // In Progress: LAT-2 Urgent, LAT-8 High, LAT-5 no priority.
    assert_eq!(sorted_keys(SortKey::Priority, false), ["LAT-2", "LAT-8", "LAT-5"]);
    assert_eq!(sorted_keys(SortKey::Priority, true), ["LAT-5", "LAT-8", "LAT-2"]);
}

#[test]
fn a_group_sorts_by_date_and_by_title() {
    assert_eq!(sorted_keys(SortKey::Updated, false), ["LAT-8", "LAT-2", "LAT-5"]);
    assert_eq!(sorted_keys(SortKey::Updated, true), ["LAT-5", "LAT-2", "LAT-8"]);
    assert_eq!(sorted_keys(SortKey::Created, false), ["LAT-8", "LAT-2", "LAT-5"]);
    assert_eq!(sorted_keys(SortKey::Title, false), ["LAT-2", "LAT-5", "LAT-8"]);
}

#[test]
fn equal_tasks_fall_back_to_their_key_so_the_order_never_shuffles() {
    let tasks: Vec<TaskData> = (1..=5).rev().map(|n| task(n, TaskStatus::Todo, Priority::High, None, &[], 7)).collect();
    let groups = group(&tasks, &Filters::default(), "", Sort::default(), &[TaskStatus::Todo], false);
    assert_eq!(keys(&tasks, &groups[0].tasks), ["LAT-1", "LAT-2", "LAT-3", "LAT-4", "LAT-5"]);
}

#[test]
fn rows_are_a_header_then_its_tasks_and_a_folded_group_is_only_its_header() {
    let tasks = sample();
    let groups = group(&tasks, &Filters::default(), "ada", Sort::default(), &TaskStatus::LIST_ORDER, false);
    let open = rows(&groups, &Folds::default());
    assert_eq!(open.len(), 5 + 8);
    assert_eq!(open[0], Row::Header { status: TaskStatus::InProgress, count: 3, open: true });
    let mut folds = Folds::default();
    folds.toggle(TaskStatus::InProgress);
    let folded = rows(&groups, &folds);
    assert_eq!(folded[0], Row::Header { status: TaskStatus::InProgress, count: 3, open: false });
    assert_eq!(folded.len(), 5 + 5, "the three tasks of the folded group are gone");
    folds.toggle(TaskStatus::InProgress);
    assert_eq!(rows(&groups, &folds), open);
}

fn view() -> (Vec<TaskData>, Vec<super::Group>, Vec<Row>) {
    let tasks = sample();
    let groups = group(&tasks, &Filters::default(), "ada", Sort::default(), &TaskStatus::LIST_ORDER, false);
    let r = rows(&groups, &Folds::default());
    (tasks, groups, r)
}

#[test]
fn the_cursor_moves_over_headers_and_tasks_and_stops_at_the_ends() {
    let (_, _, r) = view();
    let mut c = Cursor::default();
    c.go(&r, Move::Down);
    assert_eq!(c.row, Some(0));
    c.go(&r, Move::Down);
    assert_eq!(c.row, Some(1));
    c.go(&r, Move::Up);
    c.go(&r, Move::Up);
    assert_eq!(c.row, Some(0));
    c.go(&r, Move::Last);
    assert_eq!(c.row, Some(r.len() - 1));
    c.go(&r, Move::Down);
    assert_eq!(c.row, Some(r.len() - 1));
    c.go(&r, Move::First);
    assert_eq!(c.row, Some(0));
    let mut fresh = Cursor::default();
    fresh.go(&r, Move::Up);
    assert_eq!(fresh.row, Some(r.len() - 1), "with no cursor, up takes the last row");
    let mut empty = Cursor { row: Some(3), ..Cursor::default() };
    empty.go(&[], Move::Down);
    assert_eq!(empty.row, None);
}

#[test]
fn x_selects_the_task_under_the_cursor_and_again_unselects_it() {
    let (tasks, _, r) = view();
    let mut c = Cursor { row: Some(1), ..Cursor::default() };
    c.toggle(&r, &tasks);
    assert_eq!(c.selected.len(), 1);
    c.toggle(&r, &tasks);
    assert!(c.selected.is_empty());
}

#[test]
fn x_on_a_header_selects_its_group_and_takes_it_out_again_when_all_were_in() {
    let (tasks, _, r) = view();
    let mut c = Cursor { row: Some(0), ..Cursor::default() };
    c.toggle(&r, &tasks);
    assert_eq!(c.selected.len(), 3, "the three tasks of In Progress");
    c.toggle(&r, &tasks);
    assert!(c.selected.is_empty());
    // One already selected: x completes the group instead of clearing it.
    c.row = Some(1);
    c.toggle(&r, &tasks);
    c.row = Some(0);
    c.toggle(&r, &tasks);
    assert_eq!(c.selected.len(), 3);
}

#[test]
fn select_all_takes_every_shown_task_even_in_a_folded_group() {
    let (tasks, groups, _) = view();
    let mut c = Cursor::default();
    c.select_all(&groups, &tasks);
    assert_eq!(c.selected.len(), 8);
}

#[test]
fn escape_clears_the_selection_first() {
    let (tasks, groups, _) = view();
    let mut c = Cursor::default();
    assert!(!c.clear(), "nothing to clear");
    c.select_all(&groups, &tasks);
    assert!(c.clear() && c.selected.is_empty());
}

#[test]
fn a_field_key_acts_on_the_selection_else_on_the_task_under_the_cursor() {
    let (tasks, _, r) = view();
    let mut c = Cursor { row: Some(1), ..Cursor::default() };
    let under: Vec<SharedString> = c.acting_on(&r, &tasks);
    assert_eq!(under.len(), 1);
    c.row = Some(0);
    assert!(c.acting_on(&r, &tasks).is_empty(), "a header is not a task");
    c.selected.insert("t3".into());
    c.selected.insert("t4".into());
    assert_eq!(c.acting_on(&r, &tasks), ["t3", "t4"].map(SharedString::from));
}

#[test]
fn the_cursor_follows_its_task_when_the_rows_reorder() {
    let (mut tasks, _, r) = view();
    let mut c = Cursor { row: Some(1), ..Cursor::default() };
    let id = tasks[c.task(&r).unwrap()].id.clone();
    // Its priority drops to the lowest: it moves down its group.
    tasks.iter_mut().find(|t| t.id == id).unwrap().priority = Priority::None;
    tasks.iter_mut().find(|t| t.id == id).unwrap().updated_at = 0;
    let groups = group(&tasks, &Filters::default(), "ada", Sort::default(), &TaskStatus::LIST_ORDER, false);
    let after = rows(&groups, &Folds::default());
    c.follow(&after, &tasks, Some(&id));
    assert_eq!(tasks[c.task(&after).unwrap()].id, id);
    assert_ne!(c.row, Some(1));
}

#[test]
fn a_cursor_whose_task_left_the_rows_stays_in_range() {
    let (tasks, _, r) = view();
    let mut c = Cursor { row: Some(r.len() - 1), ..Cursor::default() };
    let shorter = &r[..3];
    c.follow(shorter, &tasks, Some(&"gone".into()));
    assert_eq!(c.row, Some(2));
    c.follow(&[], &tasks, None);
    assert_eq!(c.row, None);
}

#[test]
fn a_chip_cycles_through_its_options_and_then_off() {
    let options = ["a", "b", "c"];
    assert_eq!(cycle(None, &options), Some("a"));
    assert_eq!(cycle(Some("a"), &options), Some("b"));
    assert_eq!(cycle(Some("c"), &options), None, "after the last, off");
    assert_eq!(cycle(Some("zzz"), &options), Some("a"), "a value no longer offered starts over");
    assert_eq!(cycle::<&str>(None, &[]), None);
    assert_eq!(cycle(Some("a"), &[]), None);
}

#[test]
fn a_filter_choice_sets_the_value_and_the_same_choice_again_clears_it() {
    let base = Filters::default();
    let high = base.with(&Change::Priority(Priority::High));
    assert_eq!(high.priority, Some(Priority::High));
    assert_eq!(high.with(&Change::Priority(Priority::Low)).priority, Some(Priority::Low), "another value replaces it");
    assert_eq!(high.with(&Change::Priority(Priority::High)).priority, None, "the same value clears it");
    let ui = Label::new("ui", 1);
    let f = base.with(&Change::ToggleLabel(ui.clone()));
    assert_eq!(f.label.as_deref(), Some("ui"));
    assert_eq!(f.with(&Change::ToggleLabel(ui)).label, None);
}

#[test]
fn a_filter_by_assignee_takes_the_name_and_unassigned_clears_it() {
    let sam = Assignee::Person { name: "Sam".into() };
    let f = Filters::default().with(&Change::Assignee(Some(sam.clone())));
    assert_eq!(f.assignee.as_deref(), Some("Sam"));
    assert_eq!(f.with(&Change::Assignee(None)).assignee, None);
    assert_eq!(f.with(&Change::Assignee(Some(sam))).assignee, None);
    assert_eq!(f.with(&Change::Status(TaskStatus::Done)), f, "a status is no filter");
}
