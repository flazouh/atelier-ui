use std::{cell::Cell, rc::Rc};

use gpui_kit::{IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use crate::{
    theme::{Appearance, set_appearance},
    update_button::UpdateButton,
};

struct Page(UpdateButton);
impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        div().p(px(20.)).child(self.0.clone())
    }
}

fn pressed_in(make: impl FnOnce(UpdateButton) -> UpdateButton, cx: &mut TestAppContext) -> (Rc<Cell<u32>>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let count = Rc::new(Cell::new(0));
    let seen = count.clone();
    let button = make(UpdateButton::new("u")).on_click(move |_, _| seen.set(seen.get() + 1));
    let (_, cx) = cx.add_window_view(|_, _| Page(button));
    cx.simulate_resize(gpui_kit::size(px(400.), px(200.)));
    cx.run_until_parked();
    (count, cx)
}

/// A key pressed and let go: gpui turns Enter or Space on a focused element into a click on the release.
fn tap(cx: &mut VisualTestContext, key: &str) {
    let keystroke = gpui_kit::Keystroke::parse(key).expect("a key");
    cx.simulate_event(gpui_kit::KeyDownEvent { keystroke: keystroke.clone(), is_held: false, prefer_character_input: false });
    cx.simulate_event(gpui_kit::KeyUpEvent { keystroke });
    cx.run_until_parked();
}

fn click(cx: &mut VisualTestContext) {
    let at = cx.debug_bounds("update-button").expect("drawn").center();
    cx.simulate_click(at, Modifiers::default());
    cx.run_until_parked();
}

#[gpui_kit::test]
fn a_click_fires_only_when_ready(cx: &mut TestAppContext) {
    let (count, c) = pressed_in(|b| b.ready("Update to v0.1.9"), cx);
    click(c);
    assert_eq!(count.get(), 1);
    let (count, c) = pressed_in(|b| b.downloading(0.4, "Updating 40%"), cx);
    click(c);
    assert_eq!(count.get(), 0, "downloading");
    let (count, c) = pressed_in(|b| b.restarting("Restarting…"), cx);
    click(c);
    assert_eq!(count.get(), 0, "restarting");
}

#[gpui_kit::test]
fn enter_and_space_press_it_when_ready(cx: &mut TestAppContext) {
    let (count, c) = pressed_in(|b| b.ready("Update to v0.1.9"), cx);
    c.update(|window, cx| window.focus_next(cx));
    c.update(|window, _| window.refresh());
    c.run_until_parked();
    tap(c, "enter");
    assert_eq!(count.get(), 1, "enter");
    tap(c, "space");
    assert_eq!(count.get(), 2, "space");
}

#[gpui_kit::test]
fn the_other_states_are_no_stop_in_the_tab_order(cx: &mut TestAppContext) {
    let (count, c) = pressed_in(|b| b.downloading(0.4, "Updating 40%"), cx);
    c.update(|window, cx| window.focus_next(cx));
    c.run_until_parked();
    tap(c, "enter");
    assert_eq!(count.get(), 0);
}
