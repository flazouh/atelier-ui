use std::{cell::Cell, rc::Rc};

use gpui_kit::{AssetSource, IntoElement, ParentElement, Render, Styled, TestAppContext, div, px};

use super::{
    HERO_B_PATH, HERO_PATH, WelcomePage,
    consts::{ACTION_AT, ACTION_SECONDS, INTRO_SECONDS, NAME_AT, SWEEP_AT, SWEEP_SECONDS, ZOOM_FROM},
    helpers::{frame, glyph, intro_done, sweep_weight},
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
    assert_eq!((f.picture, f.mark, f.name, f.line, f.action), (0., 0., 0., 0., 0.));
    assert!((f.zoom - ZOOM_FROM).abs() < 1e-3, "zoomed in to {}, got {}", ZOOM_FROM, f.zoom);
    assert_eq!(f.breath, 0.);
    assert_eq!(f.sweep, None);
    assert!(!intro_done(0.));
}

#[test]
fn the_parts_come_in_one_after_the_other() {
    let at = |t: f32| frame(t, false);
    let early = at(NAME_AT + 0.2);
    assert!(early.mark > early.name, "the mark leads the name");
    assert!(early.name > early.line, "the name leads the line");
    assert!(early.line >= early.action, "the line leads the button");
    let late = at(ACTION_AT + ACTION_SECONDS / 2.);
    assert!(late.action > 0. && late.action < 1., "the button is on its way in");
    assert_eq!((late.mark, late.name, late.line), (1., 1., 1.), "the rest already stand");
}

#[test]
fn the_letters_of_the_name_start_at_the_left() {
    let t = NAME_AT + 0.3;
    assert!(glyph(t, 0., false) > glyph(t, 0.5, false), "the first letter leads the middle one");
    assert!(glyph(t, 0.5, false) > glyph(t, 1., false), "the middle one leads the last");
    assert_eq!(glyph(INTRO_SECONDS, 1., false), 1., "the last letter stands by the end");
}

#[test]
fn the_glint_runs_over_the_name_once_everything_stands_and_then_is_gone() {
    assert_eq!(frame(SWEEP_AT - 0.1, false).sweep, None);
    let start = frame(SWEEP_AT, false).sweep.expect("the glint starts");
    let end = frame(SWEEP_AT + SWEEP_SECONDS - 0.01, false).sweep.expect("the glint is still running");
    assert!(start < 0. && end > 1., "it runs from off the left edge to off the right");
    assert_eq!(frame(SWEEP_AT + SWEEP_SECONDS, false).sweep, None);
    assert!(sweep_weight(0.5, 0.5) > sweep_weight(0.5, 0.7), "a letter is lit most when the glint is on it");
}

#[test]
fn once_the_opening_is_over_everything_stands_and_only_the_breath_moves() {
    let f = frame(INTRO_SECONDS, false);
    assert_eq!((f.picture, f.mark, f.name, f.line, f.action), (1., 1., 1., 1., 1.));
    assert!(intro_done(INTRO_SECONDS));
    let (a, b) = (frame(INTRO_SECONDS + 1., false), frame(INTRO_SECONDS + 3., false));
    assert_ne!(a.breath, b.breath, "the second picture breathes");
    assert!((a.zoom - 1.).abs() < 0.1 && (b.zoom - 1.).abs() < 0.1, "the picture only swings a little");
}

#[test]
fn under_reduce_motion_the_page_is_at_rest_from_the_first_frame() {
    let f = frame(0., true);
    assert_eq!((f.picture, f.mark, f.name, f.line, f.action, f.zoom, f.breath), (1., 1., 1., 1., 1., 1., 0.));
    assert_eq!(f.sweep, None);
    assert_eq!(glyph(0., 1., true), 1.);
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
fn the_page_stacks_the_mark_the_name_the_line_and_the_button_in_the_middle(cx: &mut TestAppContext) {
    let (_, cx) = open(cx);
    let page = cx.debug_bounds("welcome-page").expect("the page is drawn");
    assert!(cx.debug_bounds("welcome-picture").is_some(), "the picture is drawn");
    let parts: Vec<_> = ["welcome-mark", "welcome-name", "welcome-line", "welcome-action"]
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
