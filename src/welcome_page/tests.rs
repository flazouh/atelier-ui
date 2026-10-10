use std::{cell::Cell, rc::Rc};
use gpui_kit::{IntoElement, ParentElement, Render, Styled, TestAppContext, div, px};
use super::WelcomePage;
struct Page {
    pressed: Rc<Cell<u32>>,
}
impl Render for Page {
    fn render(&mut self, _: &mut gpui_kit::Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let pressed = self.pressed.clone();
        div()
            .w(px(1280.))
            .h(px(900.))
            .child(WelcomePage::new("welcome").on_continue(move |_, _| pressed.set(pressed.get() + 1)))
    }
}
fn open(cx: &mut TestAppContext) -> (Rc<Cell<u32>>, &mut gpui_kit::VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
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
fn the_page_draws_a_band_a_title_a_sentence_and_a_button_in_that_order(cx: &mut TestAppContext) {
    let (_, cx) = open(cx);
    for name in [
        "welcome-band",
        "welcome-mark",
        "welcome-title",
        "welcome-text",
        "welcome-action",
        "welcome-bars",
    ] {
        assert!(cx.debug_bounds(name).is_some(), "{name} is drawn");
    }
    let band = cx.debug_bounds("welcome-band").unwrap();
    let title = cx.debug_bounds("welcome-title").unwrap();
    let text = cx.debug_bounds("welcome-text").unwrap();
    let action = cx.debug_bounds("welcome-action").unwrap();
    assert_eq!(f32::from(band.size.height), 420., "the band is 420 tall");
    assert!(title.bottom() <= band.bottom(), "the title stands in the band");
    assert!(band.bottom() <= text.top(), "the sentence is under the band");
    assert!(text.bottom() <= action.top(), "the button is under the sentence");
}
#[gpui_kit::test]
fn the_mark_the_title_and_the_sentence_share_the_left_edge(cx: &mut TestAppContext) {
    let (_, cx) = open(cx);
    let mark = cx.debug_bounds("welcome-mark").unwrap();
    let title = cx.debug_bounds("welcome-title").unwrap();
    let text = cx.debug_bounds("welcome-text").unwrap();
    assert_eq!(mark.left(), title.left(), "mark and title line up");
    assert_eq!(title.left(), text.left(), "the sentence lines up with the title");
}
#[gpui_kit::test]
fn the_bars_stand_at_the_foot_left_under_the_button(cx: &mut TestAppContext) {
    let (_, cx) = open(cx);
    let bars = cx.debug_bounds("welcome-bars").unwrap();
    let title = cx.debug_bounds("welcome-title").unwrap();
    assert_eq!(bars.left(), title.left(), "the bars start at the left edge");
    assert!(
        bars.top() > cx.debug_bounds("welcome-action").unwrap().bottom(),
        "the bars are under the button"
    );
    assert_eq!(f32::from(bars.size.width), 4. * 28. + 3. * 6., "four bars with a gap");
}
#[gpui_kit::test]
fn pressing_the_button_continues_once(cx: &mut TestAppContext) {
    let (pressed, cx) = open(cx);
    let button = cx.debug_bounds("welcome-continue").expect("the button is drawn");
    cx.simulate_click(button.center(), gpui_kit::Modifiers::none());
    assert_eq!(pressed.get(), 1, "one press, one call");
}
