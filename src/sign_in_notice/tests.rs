use std::{cell::Cell, rc::Rc};

use gpui_kit::{IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, Window, div};

use super::*;
use crate::menu::Lead;
use crate::theme::{Appearance, set_appearance};

#[test]
fn the_words_name_the_agent_and_its_account() {
    assert_eq!(words("Claude Code", None, &SignInState::Ready), "Claude Code is not signed in.");
    assert_eq!(words("Claude Code", Some("work"), &SignInState::Ready), "Claude Code is not signed in (account work).");
}

#[test]
fn the_words_follow_the_sign_in_as_it_goes() {
    assert_eq!(words("Cursor", None, &SignInState::Waiting), "Finish signing in to Cursor in your browser.");
    assert_eq!(
        words("Cursor", None, &SignInState::Failed("The sign-in did not finish: closed".into())),
        "Cursor is not signed in. The sign-in did not finish: closed"
    );
}

#[test]
fn on_another_host_the_words_say_where_to_sign_in_and_that_the_message_must_go_again() {
    assert_eq!(
        words("Claude Code", None, &SignInState::Elsewhere("dev-box".into())),
        "Claude Code is not signed in on dev-box. Sign in there, then send your message again."
    );
}

struct Host {
    state: SignInState,
    signed: Rc<Cell<usize>>,
    cancelled: Rc<Cell<usize>>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let signed = self.signed.clone();
        let cancelled = self.cancelled.clone();
        div().w(gpui_kit::px(520.)).child(
            SignInNotice::new("notice", "Claude Code", Lead::Monogram).state(self.state.clone()).on_sign_in(move |_, _| signed.set(signed.get() + 1))
                .on_cancel(move |_, _| cancelled.set(cancelled.get() + 1)),
        )
    }
}

fn shown(state: SignInState, cx: &mut TestAppContext) -> (Rc<Cell<usize>>, Rc<Cell<usize>>, &mut gpui_kit::VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let signed = Rc::new(Cell::new(0));
    let cancelled = Rc::new(Cell::new(0));
    let (host_signed, host_cancelled) = (signed.clone(), cancelled.clone());
    let (_, cx) = cx.add_window_view(move |_, _| Host { state, signed: host_signed, cancelled: host_cancelled });
    cx.run_until_parked();
    (signed, cancelled, cx)
}

#[gpui_kit::test]
fn the_sign_in_button_signs_in(cx: &mut TestAppContext) {
    let (signed, _, cx) = shown(SignInState::Ready, cx);
    assert!(cx.debug_bounds("sign-in-notice").is_some());
    let button = cx.debug_bounds("sign-in-button").expect("a notice that can be signed in has its button");
    cx.simulate_click(button.center(), Modifiers::default());
    assert_eq!(signed.get(), 1);
}

#[gpui_kit::test]
fn a_failed_sign_in_can_be_tried_again(cx: &mut TestAppContext) {
    let (signed, _, cx) = shown(SignInState::Failed("no".into()), cx);
    let button = cx.debug_bounds("sign-in-button").expect("the button stays");
    cx.simulate_click(button.center(), Modifiers::default());
    assert_eq!(signed.get(), 1);
}

#[gpui_kit::test]
fn while_the_browser_waits_the_button_does_nothing(cx: &mut TestAppContext) {
    let (signed, _, cx) = shown(SignInState::Waiting, cx);
    let button = cx.debug_bounds("sign-in-button").expect("the button shows that it waits");
    cx.simulate_click(button.center(), Modifiers::default());
    assert_eq!(signed.get(), 0, "a second sign-in would open a second browser");
}

#[gpui_kit::test]
fn on_another_host_there_is_no_button(cx: &mut TestAppContext) {
    let (_, _, cx) = shown(SignInState::Elsewhere("dev-box".into()), cx);
    assert!(cx.debug_bounds("sign-in-notice").is_some());
    assert!(cx.debug_bounds("sign-in-button").is_none());
}

#[gpui_kit::test]
fn while_the_browser_waits_the_reader_can_cancel(cx: &mut TestAppContext) {
    let (signed, cancelled, cx) = shown(SignInState::Waiting, cx);
    let cancel = cx.debug_bounds("sign-in-cancel").expect("a wait that never ends has a way out");
    cx.simulate_click(cancel.center(), Modifiers::default());
    assert_eq!((cancelled.get(), signed.get()), (1, 0));
}

#[gpui_kit::test]
fn there_is_nothing_to_cancel_before_the_browser_opens(cx: &mut TestAppContext) {
    let (_, _, cx) = shown(SignInState::Ready, cx);
    assert!(cx.debug_bounds("sign-in-cancel").is_none());
}

#[gpui_kit::test]
fn after_a_failed_try_there_is_nothing_to_cancel(cx: &mut TestAppContext) {
    let (_, _, cx) = shown(SignInState::Failed("no".into()), cx);
    assert!(cx.debug_bounds("sign-in-cancel").is_none());
}
