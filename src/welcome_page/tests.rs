use std::{cell::Cell, rc::Rc};

use gpui_kit::{AssetSource, IntoElement, ParentElement, Render, Styled, TestAppContext, div, px};

use super::{
    HERO_PATH, WelcomePage,
    consts::{ACTION_SECONDS, ACTION_WAIT, PICTURE_SECONDS, TEXT_STEP, TEXT_WAIT, TITLE_AT, TITLE_STEP},
    helpers::{action, all_in, moving, picture, shown, text_pace, title_pace},
};
use crate::Assets;

const TITLE: &str = "Welcome to Atelier.";
const TEXT: &str = "Your workshop for crafting.";

#[test]
fn the_assets_serve_the_picture() {
    let bytes = Assets.load(HERO_PATH).expect("the source answers").expect("the picture is served");
    assert!(bytes.starts_with(&[0xFF, 0xD8]), "a jpeg");
}

#[test]
fn the_picture_shows_nothing_until_it_is_loaded_then_fades_in_before_the_first_word() {
    assert_eq!(picture(None, false), 0., "not loaded, not shown");
    assert_eq!(picture(None, true), 0., "nor under Reduce Motion");
    assert_eq!(picture(Some(0.), false), 0.);
    let half = picture(Some(PICTURE_SECONDS / 2.), false);
    assert!(half > 0. && half < 1., "it is fading in");
    assert_eq!(picture(Some(PICTURE_SECONDS), false), 1.);
    assert!(PICTURE_SECONDS <= TITLE_AT, "it stands before the title starts");
    assert_eq!(picture(Some(0.), true), 1., "under Reduce Motion it shows at once");
}

#[test]
fn before_the_first_word_nothing_is_in_and_the_button_is_hidden() {
    assert_eq!(shown(TITLE, title_pace(), 0., false), 0);
    assert_eq!(shown(TITLE, title_pace(), TITLE_AT - 0.01, false), 0);
    assert_eq!(shown(TEXT, text_pace(TITLE), TITLE_AT, false), 0);
    assert_eq!(action(TITLE, TEXT, 0., false), 0.);
    assert!(moving(TITLE, TEXT, 0., false));
}

#[test]
fn the_title_comes_in_a_word_at_a_time_and_never_cuts_inside_a_word() {
    let at = |n: usize| shown(TITLE, title_pace(), TITLE_AT + TITLE_STEP * n as f32 + 0.001, false);
    assert_eq!(&TITLE[..at(0)], "Welcome ");
    assert_eq!(&TITLE[..at(1)], "Welcome to ");
    assert_eq!(at(2), TITLE.len(), "the last word brings the end");
    assert_eq!(at(40), TITLE.len());
}

#[test]
fn the_line_starts_after_the_title_and_comes_in_faster() {
    let pace = text_pace(TITLE);
    let title_done = TITLE_AT + TITLE_STEP * 2.;
    assert!((pace.at - (title_done + TEXT_WAIT)).abs() < 1e-5, "it waits a moment after the title's last word");
    assert_eq!(shown(TEXT, pace, title_done, false), 0, "no word of the line while the title ends");
    let at = |n: usize| shown(TEXT, pace, pace.at + TEXT_STEP * n as f32 + 0.001, false);
    assert_eq!(&TEXT[..at(0)], "Your ");
    assert_eq!(&TEXT[..at(1)], "Your workshop ");
    assert_eq!(at(3), TEXT.len());
    assert!(TEXT_STEP < TITLE_STEP);
    assert!((all_in(TITLE, TEXT) - (pace.at + TEXT_STEP * 3.)).abs() < 1e-5);
}

#[test]
fn the_button_comes_after_the_last_word_and_then_nothing_moves() {
    let done = all_in(TITLE, TEXT);
    assert!(action(TITLE, TEXT, done + ACTION_WAIT, false) < 1e-3, "it waits after the last word");
    let half = action(TITLE, TEXT, done + ACTION_WAIT + ACTION_SECONDS / 2., false);
    assert!(half > 0. && half < 1., "it is on its way in");
    let rest = done + ACTION_WAIT + ACTION_SECONDS + 0.001;
    assert_eq!(action(TITLE, TEXT, rest, false), 1.);
    assert!(!moving(TITLE, TEXT, rest, false), "no more frames once it stands");
}

#[test]
fn under_reduce_motion_everything_stands_from_the_first_frame() {
    assert_eq!(shown(TITLE, title_pace(), 0., true), TITLE.len());
    assert_eq!(shown(TEXT, text_pace(TITLE), 0., true), TEXT.len());
    assert_eq!(action(TITLE, TEXT, 0., true), 1.);
    assert!(!moving(TITLE, TEXT, 0., true));
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
fn the_mark_is_top_left_the_title_and_its_line_are_centred_and_the_button_is_alone_bottom_right(cx: &mut TestAppContext) {
    let (_, cx) = open(cx);
    let page = cx.debug_bounds("welcome-page").expect("the page is drawn");
    assert!(cx.debug_bounds("welcome-picture").is_some(), "the picture is drawn");
    let [mark, title, text, button] = ["welcome-mark", "welcome-title", "welcome-text", "welcome-continue"]
        .map(|name| cx.debug_bounds(name).unwrap_or_else(|| panic!("{name} is drawn")));
    assert_eq!((f32::from(mark.left() - page.left()), f32::from(mark.top() - page.top())), (64., 56.), "the mark is top left");
    for (name, part) in [("title", title), ("line", text)] {
        let off = f32::from(part.center().x - page.center().x);
        assert!(off.abs() <= 1., "the {name} is centred across, off by {off}");
    }
    assert!(title.bottom() <= text.top(), "the line is under the title");
    let middle = f32::from((title.top() + text.bottom()) / 2. - page.center().y);
    assert!(middle < 0. && middle > -40., "the two stand in the middle, lifted a little: {middle}");
    assert_eq!((f32::from(page.right() - button.right()), f32::from(page.bottom() - button.bottom())), (64., 56.), "the button is bottom right");
    assert!(button.top() > text.bottom(), "the button is under the words");
}

#[gpui_kit::test]
fn pressing_the_button_continues_once(cx: &mut TestAppContext) {
    let (pressed, cx) = open(cx);
    let button = cx.debug_bounds("welcome-continue").expect("the button is drawn");
    cx.simulate_click(button.center(), gpui_kit::Modifiers::none());
    assert_eq!(pressed.get(), 1, "one press, one call");
}
