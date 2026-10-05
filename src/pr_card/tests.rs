use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px};

use super::*;
use crate::{
    merge::{MergeMethod, first_choice},
    pr::PrState,
    theme::{Appearance, set_appearance},
};

/// Cards, one for each set of facts, logging what they report.
struct Owner {
    facts: Vec<MergeFacts>,
    log: Rc<RefCell<Vec<String>>>,
}

impl Render for Owner {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let cards = self.facts.iter().enumerate().map(|(i, facts)| {
            let (opened, merged) = (self.log.clone(), self.log.clone());
            let pr = PrChipData { number: 7, repo: "o/r".into(), title: "Fix".into(), state: PrState::Open, url: "https://x".into(), facts: None };
            PrCard::new(("card", i), pr)
                .on_open(move |_, _, _| opened.borrow_mut().push("open".into()))
                .merge(facts.clone(), first_choice(facts, None))
                .on_merge(move |action, _, _| merged.borrow_mut().push(action.word()))
        });
        div().w(px(900.)).p(px(20.)).flex().flex_col().gap(px(8.)).children(cards)
    }
}

fn open(facts: Vec<MergeFacts>, cx: &mut TestAppContext) -> (&mut gpui_kit::VisualTestContext, Rc<RefCell<Vec<String>>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (_owner, cx) = cx.add_window_view(move |_, _| Owner { facts, log: seen });
    cx.run_until_parked();
    (cx, log)
}

/// A press on the card's merge button merges; it does not open the pull request under it.
#[gpui_kit::test]
fn the_merge_button_on_the_card_is_its_own_press(cx: &mut TestAppContext) {
    let (cx, log) = open(vec![MergeFacts { default_method: MergeMethod::Squash, ..MergeFacts::default() }], cx);
    let main = cx.debug_bounds("merge-main").expect("the card shows the button");
    cx.simulate_click(main.center(), gpui_kit::Modifiers::default());
    assert_eq!(*log.borrow(), ["Squash and merge"]);
}

/// A merged card, a closed one, and one the reader cannot merge show no button.
#[gpui_kit::test]
fn no_button_on_a_merged_closed_or_read_only_card(cx: &mut TestAppContext) {
    let facts = vec![
        MergeFacts { state: PullState::Merged, ..MergeFacts::default() },
        MergeFacts { state: PullState::Closed, ..MergeFacts::default() },
        MergeFacts { rights: Rights::Cannot, ..MergeFacts::default() },
    ];
    let (cx, _) = open(facts, cx);
    assert!(cx.debug_bounds("merge-main").is_none(), "no card shows a button");
}
