use std::{cell::RefCell, rc::Rc};

use gpui_kit::{AppContext, TestAppContext};

use super::{TaskBoard, TaskBoardEvent};
use crate::{
    task_edit::Change,
    task_model::{TaskData, TaskStatus},
    theme::{Appearance, set_appearance},
};

fn tasks() -> Vec<TaskData> {
    vec![
        TaskData::new("a", "LAT-1", "One", TaskStatus::Todo),
        TaskData::new("b", "LAT-2", "Two", TaskStatus::Todo),
        TaskData::new("c", "LAT-3", "Three", TaskStatus::Done),
    ]
}

fn count(board: &TaskBoard, status: TaskStatus) -> usize {
    board
        .columns()
        .iter()
        .find(|c| c.status == status)
        .map_or(0, |c| c.tasks.len())
}

/// Dropping a card on another column gives the card that status, moves it between the columns, and tells
/// the owner. A drop on its own column changes nothing.
#[gpui_kit::test]
fn a_card_dropped_on_another_column_takes_its_status(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let board = cx.new(|cx| TaskBoard::new("Alex", cx));
    let seen = Rc::new(RefCell::new(Vec::new()));
    let log = seen.clone();
    let _sub = cx.update(|cx| {
        cx.subscribe(&board, move |_, event: &TaskBoardEvent, _| {
            log.borrow_mut().push(event.clone())
        })
    });
    board.update(cx, |b, cx| b.set_tasks(tasks(), 100, cx));
    board.update(cx, |b, cx| {
        assert_eq!(
            (count(b, TaskStatus::Todo), count(b, TaskStatus::InProgress)),
            (2, 0)
        );
        b.drop_card(&"a".into(), TaskStatus::Todo, cx);
    });
    assert!(
        seen.borrow().is_empty(),
        "a drop on its own column is nothing"
    );
    board.update(cx, |b, cx| {
        b.drop_card(&"a".into(), TaskStatus::InProgress, cx)
    });
    board.read_with(cx, |b, _| {
        assert_eq!(
            (count(b, TaskStatus::Todo), count(b, TaskStatus::InProgress)),
            (1, 1)
        );
        assert_eq!(b.tasks()[0].status, TaskStatus::InProgress);
    });
    assert_eq!(
        *seen.borrow(),
        vec![TaskBoardEvent::Changed {
            ids: vec!["a".into()],
            change: Change::Status(TaskStatus::InProgress)
        }]
    );
}

/// A board in a window, with the keyboard: arrows and Enter move and open, `]` shifts the status and the
/// cursor stays on the card.
#[gpui_kit::test]
fn the_keyboard_moves_between_cards_opens_one_and_shifts_its_status(cx: &mut TestAppContext) {
    use gpui_kit::{Focusable, KeyDownEvent, KeyUpEvent, Keystroke};
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let (board, cx) = cx.add_window_view(|_, cx| {
        let mut b = TaskBoard::new("Alex", cx);
        b.set_tasks(tasks(), 100, cx);
        b
    });
    let seen = Rc::new(RefCell::new(Vec::new()));
    let log = seen.clone();
    let _sub = cx.update(|_, cx| {
        cx.subscribe(&board, move |_, event: &TaskBoardEvent, _| {
            log.borrow_mut().push(event.clone())
        })
    });
    cx.update(|window, cx| window.focus(&board.focus_handle(cx), cx));
    let press = |cx: &mut gpui_kit::VisualTestContext, key: &str| {
        let keystroke = Keystroke::parse(key).unwrap();
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
    };
    press(cx, "down");
    assert_eq!(
        board.read_with(cx, |b, _| b.cursor()),
        Some((1, 0)),
        "the first card of the first column with cards (Todo)"
    );
    press(cx, "j");
    assert_eq!(board.read_with(cx, |b, _| b.cursor()), Some((1, 1)));
    press(cx, "k");
    press(cx, "]");
    assert_eq!(
        board.read_with(cx, |b, _| b.cursor()),
        Some((2, 0)),
        "the cursor went with the card to In Progress"
    );
    assert_eq!(
        *seen.borrow(),
        vec![TaskBoardEvent::Changed {
            ids: vec!["a".into()],
            change: Change::Status(TaskStatus::InProgress)
        }]
    );
    press(cx, "[");
    assert_eq!(
        board.read_with(cx, |b, _| b.tasks()[0].status),
        TaskStatus::Todo,
        "and back"
    );
    press(cx, "right");
    assert_eq!(
        board.read_with(cx, |b, _| b.cursor()),
        Some((4, 0)),
        "right skips the empty columns"
    );
    press(cx, "enter");
    assert_eq!(
        seen.borrow().last(),
        Some(&TaskBoardEvent::Open("c".into())),
        "Enter opens the card under the cursor"
    );
}
