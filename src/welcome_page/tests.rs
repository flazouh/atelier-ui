use std::{cell::Cell, rc::Rc};

use gpui_kit::{AssetSource, IntoElement, ParentElement, Render, Styled, TestAppContext, div, px};

use super::{
    HERO_B_PATH, HERO_PATH, WelcomePage,
    consts::{ACTION_AT, ACTION_SECONDS, INTRO_SECONDS, LINE_AT, ZOOM_FROM},
    helpers::{frame, intro_done},
};
use crate::Assets;

#[test]
fn the_assets_serve_both_pictures() {
    for path in [HERO_PATH, HERO_B_PATH] {
        let bytes = Assets.load(path).expect("the source answers").expect("the picture is served");
        assert!(bytes.starts_with(&[0xFF, 0xD8]), "{path} is a jpeg");
    }
}

#[test]
fn on_the_first_frame_everything_is_hidden_and_the_picture_is_zoomed_in() {
    let f = frame(0., false);
    assert_eq!((f.picture, f.mark, f.line, f.action), (0., 0., 0., 0.));
    assert!((f.zoom - ZOOM_FROM).abs() < 1e-3, "zoomed in to {}, got {}", ZOOM_FROM, f.zoom);
    assert_eq!(f.breath, 0.);
    assert!(!intro_done(0.));
}

#[test]
fn the_parts_come_in_one_after_the_other() {
    let at = |t: f32| frame(t, false);
    let early = at(LINE_AT + 0.2);
    assert!(early.mark > early.line, "the mark leads the line");
    assert!(early.line > early.action, "the line leads the button");
    let late = at(ACTION_AT + ACTION_SECONDS / 2.);
    assert!(late.action > 0. && late.action < 1., "the button is on its way in");
    assert_eq!((late.mark, late.line), (1., 1.), "the rest already stand");
}

#[test]
fn once_the_opening_is_over_everything_stands_and_only_the_breath_moves() {
    let f = frame(INTRO_SECONDS, false);
    assert_eq!((f.picture, f.mark, f.line, f.action), (1., 1., 1., 1.));
    assert!(intro_done(INTRO_SECONDS));
    let (a, b) = (frame(INTRO_SECONDS + 1., false), frame(INTRO_SECONDS + 2., false));
    assert_ne!(a.breath, b.breath, "the second picture breathes");
    assert!((a.zoom - 1.).abs() < 0.1 && (b.zoom - 1.).abs() < 0.1, "the picture only swings a little");
}

#[test]
fn under_reduce_motion_the_page_is_at_rest_from_the_first_frame() {
    let f = frame(0., true);
    assert_eq!((f.picture, f.mark, f.line, f.action, f.zoom, f.breath), (1., 1., 1., 1., 1., 0.));
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
fn the_page_stacks_the_mark_the_line_and_the_button_in_the_middle(cx: &mut TestAppContext) {
    let (_, cx) = open(cx);
    let page = cx.debug_bounds("welcome-page").expect("the page is drawn");
    assert!(cx.debug_bounds("welcome-picture").is_some(), "the picture is drawn");
    let parts: Vec<_> = ["welcome-mark", "welcome-line", "welcome-action"]
        .iter()
        .map(|name| cx.debug_bounds(name).unwrap_or_else(|| panic!("{name} is drawn")))
        .collect();
    for pair in parts.windows(2) {
        assert!(pair[0].bottom() <= pair[1].top(), "each part stands under the one before");
    }
    for part in &parts {
        let off = f32::from(part.center().x - page.center().x);
        assert!(off.abs() <= 1., "a part is centred, off by {off}");
    }
}

#[gpui_kit::test]
fn pressing_the_button_continues_once(cx: &mut TestAppContext) {
    let (pressed, cx) = open(cx);
    let button = cx.debug_bounds("welcome-continue").expect("the button is drawn");
    cx.simulate_click(button.center(), gpui_kit::Modifiers::none());
    assert_eq!(pressed.get(), 1, "one press, one call");
}
