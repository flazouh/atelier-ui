use gpui_kit::{Entity, Modifiers, TestAppContext, VisualTestContext, point, size};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn a_collapsed_stack_shares_one_row_and_peeks_8px_and_narrows_12px_a_place() {
    let g = Geometry::new(vec![77., 70., 77.]);
    assert_eq!(g.cell(), 77., "the row is the tallest card");
    assert_eq!(g.height(false), 12. + 77. + 8. + 8. + 36. + 12., "cards, pb-2, mt-2, footer, padding");
    let (t0, h0) = g.card(0, 0.);
    let (t1, _) = g.card(1, 0.);
    let (t2, _) = g.card(2, 0.);
    assert_eq!((t0 - t1, t1 - t2), (8., 8.), "each place back is 8px lower");
    assert_eq!(h0, 77.);
    assert_eq!((Geometry::inset(0, 0.), Geometry::inset(1, 0.), Geometry::inset(2, 0.)), (0., 12., 24.));
}

#[test]
fn an_open_stack_is_a_list_of_the_cards_4px_apart() {
    let g = Geometry::new(vec![77., 70., 77.]);
    assert_eq!(g.list(), 77. + 70. + 77. + 8.);
    assert_eq!(g.height(true), 12. + g.list() + 8. + 36. + 12.);
    let (t0, h0) = g.card(0, 1.);
    let (t1, h1) = g.card(1, 1.);
    assert_eq!((t0 - t1, h1), (77. + 4., 70.), "the second card is a card and a gap below the first");
    assert_eq!(h0, 77.);
    assert_eq!(Geometry::inset(2, 1.), 0., "no card is cut in when open");
    assert_eq!(t0, g.height(true) - 12., "the first card sits under the top padding");
}

#[test]
fn halfway_open_is_halfway_between_and_the_footprint_is_the_compact_stack_without_the_peek() {
    let g = Geometry::new(vec![77., 77.]);
    let (a, b, mid) = (g.card(1, 0.), g.card(1, 1.), g.card(1, 0.5));
    assert!((mid.0 - (a.0 + b.0) / 2.).abs() < 1e-3);
    assert_eq!(g.footprint(), 12. + 77. + 8. + 36. + 12.);
    assert_eq!(g.height(false) - g.footprint(), 8., "the peek stands 8px above the footprint");
    assert_eq!(Geometry::new(vec![]).footprint(), 12. + 8. + 36. + 12.);
}

fn open(items: Vec<NotificationItem>, view_all: bool, reduce: bool, cx: &mut TestAppContext) -> (Entity<NotificationStack>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
        crate::motion::clock::freeze();
    });
    let (stack, cx) = cx.add_window_view(move |_, cx| NotificationStack::new("notes", items, cx).view_all(view_all));
    cx.simulate_resize(size(gpui_kit::px(600.), gpui_kit::px(600.)));
    settle(&stack, cx);
    // A test window starts with focus on its first stop, and a keyboard as the last input: begin from a shut, unfocused stack.
    stack.update(cx, |s, cx| {
        s.has_focus = false;
        s.hovered = [false; 2];
        s.set_expanded(false, cx);
    });
    // The pointer starts out of the way, so a move onto the stack is a move into it.
    cx.simulate_mouse_move(point(gpui_kit::px(590.), gpui_kit::px(590.)), None, Modifiers::default());
    settle(&stack, cx);
    stack.update(cx, |s, cx| {
        s.hovered = [false; 2];
        s.set_expanded(false, cx);
    });
    (stack, cx)
}

fn settle(stack: &Entity<NotificationStack>, cx: &mut VisualTestContext) {
    for _ in 0..4 {
        cx.run_until_parked();
        stack.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
}

fn items(n: usize) -> Vec<NotificationItem> {
    (0..n).map(|i| NotificationItem::new(format!("n{i}"), format!("Notification {i}")).description("A line of description")).collect()
}

fn expanded(stack: &Entity<NotificationStack>, cx: &mut VisualTestContext) -> bool {
    stack.read_with(cx, |s, _| s.is_expanded())
}

fn center(cx: &mut VisualTestContext, name: &'static str) -> gpui_kit::Point<gpui_kit::Pixels> {
    cx.debug_bounds(name).unwrap_or_else(|| panic!("{name} is not drawn")).center()
}

#[gpui_kit::test]
fn the_pointer_over_the_stack_opens_it_and_leaving_shuts_it_after_a_moment(cx: &mut TestAppContext) {
    let (stack, cx) = open(items(3), false, true, cx);
    assert!(!expanded(&stack, cx));
    let at = center(cx, "notification-stack");
    cx.simulate_mouse_move(at, None, Modifiers::default());
    settle(&stack, cx);
    assert!(expanded(&stack, cx), "a pointer over it opens it");
    cx.simulate_mouse_move(point(gpui_kit::px(590.), gpui_kit::px(10.)), None, Modifiers::default());
    settle(&stack, cx);
    cx.executor().advance_clock(std::time::Duration::from_millis(100));
    settle(&stack, cx);
    assert!(!expanded(&stack, cx), "and away from it shuts it");
}

#[gpui_kit::test]
fn a_press_opens_a_shut_stack_then_shuts_it_or_asks_to_view_all(cx: &mut TestAppContext) {
    let (stack, cx) = open(items(2), false, true, cx);
    let heard = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let log = heard.clone();
    let sub = cx.update(|_, cx| {
        cx.subscribe(&stack, move |_, event: &NotificationEvent, _| {
            log.borrow_mut().push(match event {
                NotificationEvent::Expanded(open) => format!("expanded {open}"),
                NotificationEvent::ViewAll => "view all".to_string(),
            })
        })
    });
    stack.update(cx, |s, cx| s.press(cx));
    assert!(expanded(&stack, cx), "the first press opens it (a finger never hovers)");
    stack.update(cx, |s, cx| s.press(cx));
    assert!(!expanded(&stack, cx), "with no view all, the second shuts it");
    assert_eq!(*heard.borrow(), ["expanded true", "expanded false"]);
    drop(sub);

    let (stack, cx) = open(items(2), true, true, cx);
    stack.update(cx, |s, cx| s.press(cx));
    let heard = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let log = heard.clone();
    let sub = cx.update(|_, cx| cx.subscribe(&stack, move |_, event: &NotificationEvent, _| if matches!(event, NotificationEvent::ViewAll) { log.borrow_mut().push("view all") }));
    stack.update(cx, |s, cx| s.press(cx));
    assert!(expanded(&stack, cx), "the open stack stays open");
    assert_eq!(*heard.borrow(), ["view all"]);
    drop(sub);
}

#[gpui_kit::test]
fn escape_shuts_it_while_it_has_focus_and_enter_presses_it(cx: &mut TestAppContext) {
    let (stack, cx) = open(items(2), false, true, cx);
    let focus = stack.read_with(cx, |s, _| s.focus.clone());
    cx.update(|window, cx| focus.focus(window, cx));
    cx.simulate_keystrokes("enter");
    settle(&stack, cx);
    assert!(expanded(&stack, cx), "Enter on a shut stack opens it");
    cx.simulate_keystrokes("escape");
    settle(&stack, cx);
    assert!(!expanded(&stack, cx), "Escape shuts it");
}

#[gpui_kit::test]
fn no_notifications_says_all_caught_up_and_only_max_visible_cards_are_drawn_while_the_badge_counts_them_all(cx: &mut TestAppContext) {
    let (_stack, cx) = open(Vec::new(), false, true, cx);
    assert!(cx.debug_bounds("notification-empty").is_some());
    assert!(cx.debug_bounds("notification-stack").is_none(), "an empty stack is not a button");

    let (stack, cx) = open(items(5), false, true, cx);
    assert!(cx.debug_bounds("notification-card-0").is_some() && cx.debug_bounds("notification-card-2").is_some());
    assert!(cx.debug_bounds("notification-card-3").is_none(), "three cards at most");
    let count = stack.read_with(cx, |s, _| s.items.len());
    assert_eq!(count, 5, "the badge says five: it counts all of them");
}

#[gpui_kit::test]
fn with_reduce_motion_the_stack_is_open_at_once_and_with_motion_it_opens_over_a_third_of_a_second(cx: &mut TestAppContext) {
    let (stack, cx) = open(items(3), false, true, cx);
    stack.update(cx, |s, cx| s.set_expanded(true, cx));
    assert_eq!(stack.read_with(cx, |s, _| s.cards.value()), 1., "reduced: no in-between");
    let (stack, cx) = open(items(3), false, false, cx);
    stack.update(cx, |s, cx| s.set_expanded(true, cx));
    assert!(stack.read_with(cx, |s, _| s.cards.value()) < 0.01, "at the start of the move");
    crate::motion::clock::advance(std::time::Duration::from_millis(160));
    let mid = stack.read_with(cx, |s, _| s.cards.value());
    assert!(mid > 0.5 && mid < 1., "the out curve is past half at half the time: {mid}");
    crate::motion::clock::advance(std::time::Duration::from_millis(400));
    assert_eq!(stack.read_with(cx, |s, _| s.cards.value()), 1.);
    assert_eq!(stack.read_with(cx, |s, _| s.background.value()), 1.);
}

#[gpui_kit::test]
fn the_label_rolls_from_notifications_to_view_all_and_the_old_words_leave_first(cx: &mut TestAppContext) {
    let (stack, cx) = open(items(2), false, false, cx);
    stack.update(cx, |s, cx| s.set_expanded(true, cx));
    let (old, leaving, roll) = stack.read_with(cx, |s, _| (s.old_label.clone(), s.leaving.value(), s.roll.value()));
    assert_eq!(old.as_deref(), Some("Notifications"));
    assert!(leaving < 0.01 && roll < 0.01);
    crate::motion::clock::advance(std::time::Duration::from_millis(150));
    assert_eq!(stack.read_with(cx, |s, _| s.leaving.value()), 1., "the old words are gone in 140 ms");
    assert!(stack.read_with(cx, |s, _| s.roll.value()) > 0., "the new ones are arriving");
}

#[gpui_kit::test]
fn the_stack_is_a_tab_stop_that_focus_next_reaches(cx: &mut TestAppContext) {
    let (stack, cx) = open(items(2), false, true, cx);
    let focus = stack.read_with(cx, |s, _| s.focus.clone());
    cx.update(|window, cx| {
        window.blur(cx);
    });
    settle(&stack, cx);
    assert!(!cx.update(|window, _| focus.is_focused(window)));
    cx.simulate_keystrokes("a");
    cx.update(|window, cx| window.focus_next(cx));
    settle(&stack, cx);
    assert!(cx.update(|window, _| focus.is_focused(window)), "the next stop after nothing is the stack");
    assert!(expanded(&stack, cx), "and focus that the keyboard brought opens it");
}
