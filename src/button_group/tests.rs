use gpui_kit::ParentElement;
use gpui_kit::Styled;
use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Context, KeyDownEvent, KeyUpEvent, Keystroke, Render, TestAppContext, VisualTestContext};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn only_the_outer_corners_round() {
    let row = |i| segment_corners(i, 3, Axis::Horizontal);
    assert_eq!(row(0), Corners { top_left: true, top_right: false, bottom_left: true, bottom_right: false });
    assert_eq!(row(1), Corners { top_left: false, top_right: false, bottom_left: false, bottom_right: false });
    assert_eq!(row(2), Corners { top_left: false, top_right: true, bottom_left: false, bottom_right: true });
    let column = |i| segment_corners(i, 2, Axis::Vertical);
    assert_eq!(column(0), Corners { top_left: true, top_right: true, bottom_left: false, bottom_right: false });
    assert_eq!(column(1), Corners { top_left: false, top_right: false, bottom_left: true, bottom_right: true });
    assert_eq!(segment_corners(0, 1, Axis::Horizontal), crate::button::ROUND, "one segment is a whole button");
}

/// A group of two segments, logging presses.
struct Group(Rc<RefCell<Vec<&'static str>>>);

impl Render for Group {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (a, b) = (self.0.clone(), self.0.clone());
        div().p(px(20.)).child(
            ButtonGroup::new("g")
                .child(Button::new("one").label("One").on_click(move |_, _, _| a.borrow_mut().push("one")))
                .child(Button::new("two").label("Two").on_click(move |_, _, _| b.borrow_mut().push("two"))),
        )
    }
}

fn press(cx: &mut VisualTestContext, key: &str) {
    let keystroke = Keystroke::parse(key).unwrap();
    cx.simulate_event(KeyDownEvent { keystroke: keystroke.clone(), is_held: false, prefer_character_input: false });
    cx.simulate_event(KeyUpEvent { keystroke });
}

/// Each segment is a Tab stop, and Enter and Space press the one focused.
#[gpui_kit::test]
fn each_segment_is_a_tab_stop_that_enter_and_space_press(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (_group, cx) = cx.add_window_view(move |_, _| Group(seen));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|window, cx| window.focus_next(cx));
    press(cx, "enter");
    cx.update(|window, cx| window.focus_next(cx));
    press(cx, "space");
    assert_eq!(*log.borrow(), ["one", "two"]);
}
