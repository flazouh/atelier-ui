use gpui_kit::Focusable;
use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Entity, TestAppContext, VisualTestContext};

use super::*;
use crate::theme::{Appearance, set_appearance};

fn open(filter: Filter, cx: &mut TestAppContext) -> (Entity<Finder>, &mut VisualTestContext, Rc<RefCell<Vec<FinderEvent>>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let (finder, cx) = cx.add_window_view(|window, cx| Finder::new("Go to file", "A file, by its name", filter, window, cx));
    let events = Rc::new(RefCell::new(Vec::new()));
    let log = events.clone();
    cx.update(|window, cx| {
        cx.subscribe(&finder, move |_, event: &FinderEvent, _| log.borrow_mut().push(event.clone())).detach();
        finder.read(cx).focus_handle(cx).focus(window, cx);
    });
    (finder, cx, events)
}

fn files() -> Vec<FinderItem> {
    ["src/observer.rs", "src/server.rs", "Cargo.toml"].into_iter().map(|p| FinderItem::new(p, "").icon(p)).collect()
}

#[gpui_kit::test]
fn typing_narrows_the_rows_best_first(cx: &mut TestAppContext) {
    let (finder, cx, events) = open(Filter::Here, cx);
    finder.update(cx, |f, cx| f.set_items(files(), cx));
    assert_eq!(finder.read_with(cx, |f, _| f.shown().to_vec()), [0, 1, 2], "no words: every row");
    cx.simulate_input("server");
    assert_eq!(finder.read_with(cx, |f, _| f.shown().to_vec()), [1, 0], "the whole word first, the toml gone");
    assert_eq!(events.borrow().last(), Some(&FinderEvent::Query("server".into())));
}

#[gpui_kit::test]
fn the_arrows_walk_and_enter_picks_escape_closes(cx: &mut TestAppContext) {
    let (finder, cx, events) = open(Filter::Here, cx);
    finder.update(cx, |f, cx| f.set_items(files(), cx));
    cx.simulate_keystrokes("down down down up enter");
    assert_eq!(events.borrow().last(), Some(&FinderEvent::Pick(1)), "down twice stops at the last row, up once");
    cx.simulate_keystrokes("escape");
    assert_eq!(events.borrow().last(), Some(&FinderEvent::Dismiss));
}

#[gpui_kit::test]
fn the_owner_filters_when_it_says_so(cx: &mut TestAppContext) {
    let (finder, cx, events) = open(Filter::Owner, cx);
    cx.simulate_input("wid");
    assert_eq!(events.borrow().last(), Some(&FinderEvent::Query("wid".into())));
    finder.update(cx, |f, cx| f.set_items(vec![FinderItem::new("zeta", "not a match, shown anyway")], cx));
    assert_eq!(finder.read_with(cx, |f, _| f.shown().to_vec()), [0], "every row the owner sets shows");
}

#[gpui_kit::test]
fn the_panel_and_its_rows_are_drawn(cx: &mut TestAppContext) {
    let (finder, cx, _) = open(Filter::Here, cx);
    finder.update(cx, |f, cx| f.set_items(files(), cx));
    for _ in 0..5 {
        cx.run_until_parked();
        finder.update(cx, |_, cx| cx.notify());
    }
    let list = cx.debug_bounds("finder-list").expect("the list is drawn");
    assert!(f32::from(list.size.height) > 3. * 36., "three rows and a heading: {:?}", list.size);
}
