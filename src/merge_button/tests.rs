use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, TestAppContext, Window, div, px};

use super::*;
use crate::{
    merge::{MergeMethod, first_choice},
    theme::{Appearance, set_appearance},
};

thread_local! {
    /// The clicks that reached the page under the button.
    static PAGE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// A merge button whose owner keeps the choice and logs what it hears.
struct Owner {
    facts: MergeFacts,
    choice: Choice,
    log: Rc<RefCell<Vec<String>>>,
}

impl Render for Owner {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (acted, chose) = (self.log.clone(), self.log.clone());
        let this = cx.entity().downgrade();
        div().id("owner-root").size_full().p(px(20.)).on_click(move |_, _, _| PAGE.with(|p| p.set(p.get() + 1))).child(
            MergeButton::new("m", self.facts.clone(), self.choice)
                .on_action(move |action, _, _| acted.borrow_mut().push(action.word()))
                .on_choice(move |choice, _, cx| {
                    chose.borrow_mut().push(format!("{:?}", choice.method));
                    this.update(cx, |o, cx| {
                        o.choice = choice;
                        cx.notify();
                    })
                    .ok();
                }),
        )
    }
}

fn open(facts: MergeFacts, cx: &mut TestAppContext) -> (&mut gpui_kit::VisualTestContext, Rc<RefCell<Vec<String>>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let choice = first_choice(&facts, None);
    let (_owner, cx) = cx.add_window_view(move |_, _| Owner { facts, choice, log: seen });
    cx.run_until_parked();
    (cx, log)
}

/// Picking another method in the menu reports it, and the button then says and does that method.
#[gpui_kit::test]
fn a_method_picked_in_the_menu_is_reported_and_used(cx: &mut TestAppContext) {
    let (cx, log) = open(MergeFacts { default_method: MergeMethod::Squash, ..MergeFacts::default() }, cx);
    let arrow = cx.debug_bounds("merge-arrow").expect("the arrow is drawn");
    cx.simulate_click(arrow.center(), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    let rebase = cx.debug_bounds("merge-method-Rebase").expect("the menu lists rebase");
    cx.simulate_click(rebase.center(), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    let main = cx.debug_bounds("merge-main").expect("the main part is drawn");
    cx.simulate_click(main.center(), gpui_kit::Modifiers::default());
    assert_eq!(*log.borrow(), ["Rebase", "Rebase and merge"], "the choice, then the press with it");
}

fn press(cx: &mut gpui_kit::VisualTestContext, key: &str) {
    let keystroke = gpui_kit::Keystroke::parse(key).unwrap();
    cx.simulate_event(gpui_kit::KeyDownEvent { keystroke: keystroke.clone(), is_held: false, prefer_character_input: false });
    cx.simulate_event(gpui_kit::KeyUpEvent { keystroke });
}

/// The action and the arrow are two Tab stops. Down on the arrow opens the menu on its first row;
/// Down walks the rows, Enter picks one, and focus goes back to the arrow.
#[gpui_kit::test]
fn the_menu_opens_and_picks_from_the_keyboard(cx: &mut TestAppContext) {
    let (cx, log) = open(MergeFacts { default_method: MergeMethod::Squash, ..MergeFacts::default() }, cx);
    cx.update(|window, cx| {
        window.focus_next(cx);
        window.focus_next(cx);
    });
    press(cx, "down");
    cx.run_until_parked();
    assert!(cx.debug_bounds("merge-method-Squash").is_some(), "Down on the arrow opens the menu");
    press(cx, "down");
    press(cx, "enter");
    cx.run_until_parked();
    assert_eq!(*log.borrow(), ["Merge"], "the second row, a merge commit, was picked");
    assert!(cx.debug_bounds("merge-method-Squash").is_none(), "and the menu closed");
    press(cx, "enter");
    cx.run_until_parked();
    assert!(cx.debug_bounds("merge-method-Squash").is_some(), "focus is back on the arrow: Enter opens it again");
    press(cx, "escape");
    cx.run_until_parked();
    assert!(cx.debug_bounds("merge-method-Squash").is_none(), "Escape closes it");
}

/// A click on the arrow while the menu is open closes it.
#[gpui_kit::test]
fn a_second_click_on_the_arrow_closes_the_menu(cx: &mut TestAppContext) {
    let (cx, _) = open(MergeFacts::default(), cx);
    let arrow = cx.debug_bounds("merge-arrow").expect("the arrow is drawn");
    cx.simulate_click(arrow.center(), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert!(cx.debug_bounds("merge-auto").is_some() || cx.debug_bounds("merge-delete-branch").is_some(), "the menu opened");
    cx.simulate_click(arrow.center(), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert!(cx.debug_bounds("merge-delete-branch").is_none(), "the second click closed it");
}

/// A press outside the menu closes it and the page under it hears nothing of the press.
#[gpui_kit::test]
fn a_press_outside_the_menu_closes_it_and_reaches_nothing_else(cx: &mut TestAppContext) {
    let (cx, log) = open(MergeFacts { default_method: MergeMethod::Squash, ..MergeFacts::default() }, cx);
    let arrow = cx.debug_bounds("merge-arrow").expect("the arrow is drawn");
    cx.simulate_click(arrow.center(), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert!(cx.debug_bounds("merge-delete-branch").is_some(), "the menu is open");
    let _ = &log;
    let pages_before = PAGE.with(|p| p.get());
    cx.simulate_click(gpui_kit::point(px(5.), px(500.)), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert!(cx.debug_bounds("merge-delete-branch").is_none(), "the press outside closed it");
    assert_eq!(PAGE.with(|p| p.get()), pages_before, "and went no further");
}
