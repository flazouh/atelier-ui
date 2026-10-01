use std::{cell::RefCell, rc::Rc};

use gpui_kit::TestAppContext;

use super::*;
use crate::{
    merge::{MergeMethod, first_choice},
    theme::{Appearance, set_appearance},
};

/// A press in the box reports the action with the squash commit's words, as the box was filled.
#[gpui_kit::test]
fn a_squash_press_carries_the_commit_words(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let facts = MergeFacts { default_method: MergeMethod::Squash, ..MergeFacts::default() };
    let choice = first_choice(&facts, None);
    let (merge_box, cx) = cx.add_window_view(|window, cx| MergeBox::new(facts, choice, "Fix the abort", "Detach the stream.", window, cx));
    let events = Rc::new(RefCell::new(Vec::new()));
    let log = events.clone();
    cx.update(|_, cx| cx.subscribe(&merge_box, move |_, event: &MergeBoxEvent, _| log.borrow_mut().push(event.clone())).detach());
    cx.update(|_, cx| merge_box.update(cx, |b, cx| b.act(Action::Merge(MergeMethod::Squash), cx)));
    assert_eq!(
        events.borrow().as_slice(),
        [MergeBoxEvent::Act {
            action: Action::Merge(MergeMethod::Squash),
            choice,
            title: "Fix the abort".into(),
            message: "Detach the stream.".into(),
        }]
    );
    cx.update(|_, cx| merge_box.update(cx, |b, cx| b.merged(true, cx)));
    assert_eq!(merge_box.read_with(cx, |b, _| b.facts.state), PullState::Merged);
}
