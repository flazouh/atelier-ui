use std::{cell::Cell, rc::Rc};

use gpui_kit::{AssetSource, IntoElement, ParentElement, Render, Styled, TestAppContext, div, px};

use super::{
    HERO_PATH, WelcomePage,
    consts::{ACTION_SECONDS, ACTION_WAIT, WORD_STEP, WORDS_AT},
    helpers::{action, moving, shown, words_done},
};
use crate::Assets;

const TEXT: &str = "Welcome to atelier, your workshop.";

#[test]
fn the_assets_serve_the_picture() {
    let bytes = Assets.load(HERO_PATH).expect("the source answers").expect("the picture is served");
    assert!(bytes.starts_with(&[0xFF, 0xD8]), "a jpeg");
}

#[test]
fn before_the_first_word_nothing_is_in_and_the_button_is_hidden() {
    assert_eq!(shown(TEXT, 0., false), 0);
    assert_eq!(shown(TEXT, WORDS_AT - 0.01, false), 0);
    assert_eq!(action(TEXT, 0., false), 0.);
    assert!(moving(TEXT, 0., false));
}

#[test]
fn the_words_come_in_one_at_a_time_and_never_cut_inside_a_word() {
    let at = |n: usize| shown(TEXT, WORDS_AT + WORD_STEP * n as f32 + 0.001, false);
    assert_eq!(&TEXT[..at(0)], "Welcome ");
    assert_eq!(&TEXT[..at(1)], "Welcome to ");
    assert_eq!(&TEXT[..at(2)], "Welcome to atelier, ");
    assert_eq!(&TEXT[..at(3)], "Welcome to atelier, your ");
    assert_eq!(at(4), TEXT.len(), "the last word brings the end");
    assert_eq!(at(40), TEXT.len());
    assert!((words_done(TEXT) - (WORDS_AT + WORD_STEP * 4.)).abs() < 1e-5);
}

#[test]
fn the_button_comes_after_the_last_word_and_then_nothing_moves() {
    let done = words_done(TEXT);
    assert!(action(TEXT, done + ACTION_WAIT, false) < 1e-3, "it waits after the last word");
    let half = action(TEXT, done + ACTION_WAIT + ACTION_SECONDS / 2., false);
    assert!(half > 0. && half < 1., "it is on its way in");
    let rest = done + ACTION_WAIT + ACTION_SECONDS;
    assert_eq!(action(TEXT, rest, false), 1.);
    assert!(!moving(TEXT, rest, false), "no more frames once it stands");
}

#[test]
fn under_reduce_motion_everything_stands_from_the_first_frame() {
    assert_eq!(shown(TEXT, 0., true), TEXT.len());
    assert_eq!(action(TEXT, 0., true), 1.);
    assert!(!moving(TEXT, 0., true));
}

struct Page {
    pressed: Rc<Cell<u32>>,
}
impl Render for Page {
    fn render(&mut self, _: &mut gpui_kit::Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let pressed = self.pressed.clone();
        div().w(px(1280.)).h(px(800.)).child(WelcomePage::new("welcome").on_continue(move |_, _| pressed.set(pressed.get() + 1)))
    }
}
fn open(cx: &mut TestAppContext) -> (Rc<Cell<u32>>, &mut gpui_kit::VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let pressed = Rc::new(Cell::new(0));
    let (_, cx) = cx.add_window_view({
        let pressed = pressed.clone();
        move |_, _| Page { pressed }
    });
    cx.run_until_parked();
    (pressed, cx)
}

#[gpui_kit::test]
fn the_mark_is_top_left_the_words_stand_under_it_and_the_button_is_alone_bottom_right(cx: &mut TestAppContext) {
    let (_, cx) = open(cx);
    let page = cx.debug_bounds("welcome-page").expect("the page is drawn");
    assert!(cx.debug_bounds("welcome-picture").is_some(), "the picture is drawn");
    let [mark, title, text, button] = ["welcome-mark", "welcome-title", "welcome-text", "welcome-continue"]
        .map(|name| cx.debug_bounds(name).unwrap_or_else(|| panic!("{name} is drawn")));
    assert_eq!((f32::from(mark.left() - page.left()), f32::from(mark.top() - page.top())), (64., 56.), "the mark is top left");
    assert!(mark.bottom() < title.top(), "the title is under the mark");
    assert!(title.bottom() <= text.top(), "the line is under the title");
    assert_eq!(text.left(), mark.left(), "the line shares the mark's left edge");
    assert_eq!((f32::from(page.right() - button.right()), f32::from(page.bottom() - button.bottom())), (64., 56.), "the button is bottom right");
    assert!(button.top() > text.bottom(), "the button is under the words");
    assert!(button.left() > text.left() + (text.size.width / 2.), "and away from them, at the right");
}

#[gpui_kit::test]
fn pressing_the_button_continues_once(cx: &mut TestAppContext) {
    let (pressed, cx) = open(cx);
    let button = cx.debug_bounds("welcome-continue").expect("the button is drawn");
    cx.simulate_click(button.center(), gpui_kit::Modifiers::none());
    assert_eq!(pressed.get(), 1, "one press, one call");
}
