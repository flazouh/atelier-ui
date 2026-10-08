use gpui_kit::{IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};
use std::{cell::Cell, rc::Rc};

use crate::{
    status_bar::StatusBar,
    theme::{Appearance, set_appearance},
};

struct Bar {
    pressed: Rc<Cell<u32>>,
    wired: bool,
}

impl Render for Bar {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let pressed = self.pressed.clone();
        let bar = StatusBar::new("bar").columns(Some(200.), None);
        div()
            .w(px(800.))
            .child(if self.wired { bar.on_usage(move |_, _| pressed.set(pressed.get() + 1)) } else { bar })
    }
}

fn open(wired: bool, cx: &mut TestAppContext) -> (&mut gpui_kit::VisualTestContext, Rc<Cell<u32>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let pressed = Rc::new(Cell::new(0));
    let host = Bar { pressed: pressed.clone(), wired };
    let (_host, cx) = cx.add_window_view(move |_, _| host);
    cx.simulate_resize(size(px(800.), px(200.)));
    cx.run_until_parked();
    (cx, pressed)
}

/// With no provider reading, the bar still has a "Usage" door when the app wants one, and a press on it fires.
#[gpui_kit::test]
fn with_no_readings_the_usage_has_a_door_that_fires(cx: &mut TestAppContext) {
    let (cx, pressed) = open(true, cx);
    let door = cx.debug_bounds("status-usage").expect("the usage door is drawn");
    cx.simulate_click(door.center(), Modifiers::default());
    cx.run_until_parked();
    assert_eq!(pressed.get(), 1);
}

/// With no wish to open anything there is no door: the bar shows nothing for the usage.
#[gpui_kit::test]
fn with_no_readings_and_no_handler_there_is_no_door(cx: &mut TestAppContext) {
    let (cx, _) = open(false, cx);
    assert!(cx.debug_bounds("status-usage").is_none());
}
