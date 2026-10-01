use super::{CARD_GAP, CARD_HEIGHT, COLUMN_GAP, COLUMN_WIDTH, column_at, columns, drop_on, geometry, visible_cards};
use crate::{
    task_edit::Change,
    task_list_model::{Filters, Sort},
    task_model::{Priority, TaskData, TaskStatus},
};

fn tasks() -> Vec<TaskData> {
    let mut a = TaskData::new("a", "LAT-1", "A", TaskStatus::Todo);
    a.priority = Priority::High;
    let mut b = TaskData::new("b", "LAT-2", "B", TaskStatus::Todo);
    b.priority = Priority::Urgent;
    let c = TaskData::new("c", "LAT-3", "C", TaskStatus::Done);
    vec![a, b, c]
}

#[test]
fn a_board_has_a_column_for_every_status_in_board_order_and_keeps_the_empty_ones() {
    let cols = columns(&tasks(), &Filters::default(), "", Sort::default());
    let words: Vec<_> = cols.iter().map(|c| (c.status.words(), c.tasks.len())).collect();
    assert_eq!(words, [("Backlog", 0), ("Todo", 2), ("In Progress", 0), ("In Review", 0), ("Done", 1), ("Canceled", 0)]);
}

#[test]
fn cards_sort_inside_a_column_and_a_filter_thins_them() {
    let list = tasks();
    let cols = columns(&list, &Filters::default(), "", Sort::default());
    assert_eq!(cols[1].tasks, [1, 0], "urgent before high");
    let filtered = columns(&list, &Filters { priority: Some(Priority::High), ..Filters::default() }, "", Sort::default());
    assert_eq!(filtered[1].tasks, [0]);
    assert!(filtered[4].tasks.is_empty());
    assert_eq!(filtered.len(), 6, "the columns stay");
}

#[test]
fn dropping_a_card_on_another_column_changes_its_status_and_on_its_own_changes_nothing() {
    let list = tasks();
    assert_eq!(drop_on(&list, &"a".into(), TaskStatus::InProgress), Some(Change::Status(TaskStatus::InProgress)));
    assert_eq!(drop_on(&list, &"a".into(), TaskStatus::Todo), None);
    assert_eq!(drop_on(&list, &"zzz".into(), TaskStatus::Done), None);
}

#[test]
fn a_point_finds_its_column_and_a_gap_belongs_to_the_column_before_it() {
    let g = geometry(6);
    assert_eq!(g.len(), 6);
    assert_eq!(g.total(), 6. * COLUMN_WIDTH + 5. * COLUMN_GAP);
    assert_eq!(column_at(&g, 0.), Some(0));
    assert_eq!(column_at(&g, COLUMN_WIDTH - 1.), Some(0));
    assert_eq!(column_at(&g, COLUMN_WIDTH + 3.), Some(0), "in the gap");
    assert_eq!(column_at(&g, COLUMN_WIDTH + COLUMN_GAP), Some(1));
    assert_eq!(column_at(&g, g.total() - 1.), Some(5));
    assert_eq!((column_at(&g, -1.), column_at(&g, g.total() + 1.)), (None, None));
    assert_eq!(column_at(&geometry(0), 5.), None);
}

#[test]
fn a_column_builds_only_the_cards_in_view_and_one_more_each_side() {
    let step = CARD_HEIGHT + CARD_GAP;
    assert_eq!(visible_cards(1000, 0., 3. * step), 0..4);
    assert_eq!(visible_cards(1000, 100. * step, 3. * step), 99..104);
    assert_eq!(visible_cards(5, 0., 10_000.), 0..5);
    assert_eq!(visible_cards(0, 0., 500.), 0..0);
    assert_eq!(visible_cards(10, 50_000., 500.), 10..10, "scrolled past the end");
}

mod keyboard {
    use super::*;
    use crate::task_board_model::{BoardMove::*, Spot, move_cursor, spot_of, task_at};

    fn board() -> (Vec<TaskData>, Vec<crate::task_list_model::Group>) {
        let mut t = Vec::new();
        for (i, st) in [TaskStatus::Backlog, TaskStatus::Backlog, TaskStatus::Backlog, TaskStatus::InProgress, TaskStatus::Done, TaskStatus::Done].into_iter().enumerate() {
            t.push(TaskData::new(format!("t{i}"), format!("LAT-{i}"), format!("T{i}"), st));
        }
        let c = columns(&t, &Filters::default(), "Alex", Sort::default());
        (t, c)
    }

    #[test]
    fn the_cursor_starts_on_the_first_card_and_moves_inside_a_column() {
        let (_, c) = board();
        assert_eq!(move_cursor(&c, None, Down), Some((0, 0)));
        assert_eq!(move_cursor(&c, Some((0, 0)), Down), Some((0, 1)));
        assert_eq!(move_cursor(&c, Some((0, 2)), Down), Some((0, 2)), "stops at the end");
        assert_eq!(move_cursor(&c, Some((0, 0)), Up), Some((0, 0)));
        assert_eq!(move_cursor(&c, Some((0, 1)), Last), Some((0, 2)));
        assert_eq!(move_cursor(&c, Some((0, 2)), First), Some((0, 0)));
    }

    #[test]
    fn left_and_right_skip_empty_columns_and_clamp_the_row() {
        let (_, c) = board();
        // Backlog (3), Todo (0), In Progress (1), In Review (0), Done (2), Canceled (0).
        assert_eq!(move_cursor(&c, Some((0, 2)), Right), Some((2, 0)), "Todo is empty, so on to In Progress; row clamps to 0");
        assert_eq!(move_cursor(&c, Some((2, 0)), Right), Some((4, 0)));
        assert_eq!(move_cursor(&c, Some((4, 1)), Right), Some((4, 1)), "nothing further right");
        assert_eq!(move_cursor(&c, Some((4, 1)), Left), Some((2, 0)));
        assert_eq!(move_cursor(&c, Some((0, 1)), Left), Some((0, 1)));
    }

    #[test]
    fn a_stale_cursor_starts_over_and_an_empty_board_has_none() {
        let (t, c) = board();
        assert_eq!(move_cursor(&c, Some((9, 9)), Down), Some((0, 0)));
        let empty = columns(&t[..0], &Filters::default(), "Alex", Sort::default());
        assert_eq!(move_cursor(&empty, None, Down), None);
    }

    #[test]
    fn a_spot_finds_its_task_and_a_task_finds_its_spot() {
        let (t, c) = board();
        let spot: Spot = (4, 1);
        let index = task_at(&c, spot).unwrap();
        assert_eq!(t[index].id, "t5");
        assert_eq!(spot_of(&c, &t, &"t5".into()), Some(spot));
        assert_eq!(spot_of(&c, &t, &"nope".into()), None);
        assert_eq!(task_at(&c, (1, 0)), None);
    }
}
