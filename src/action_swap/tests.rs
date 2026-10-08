use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui_kit::{Entity, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use super::*;
use crate::{
    motion,
    theme::{Appearance, set_appearance},
};

#[test]
fn the_small_size_is_a_sm_buttons() {
    assert_eq!((HEIGHT, PAD_X, GAP, TEXT, LINE), (28., 10., 6., 11., 16.));
}

#[test]
fn a_primary_button_lightens_toward_the_page_on_hover_and_ghost_words_take_the_foreground() {
    for theme in crate::themes::all() {
        let (rest, _) = colors(SwapVariant::Primary, theme, 0.);
        let (over, _) = colors(SwapVariant::Primary, theme, 1.);
        assert_ne!(rest, over, "{}", theme.name);
        let (_, muted) = colors(SwapVariant::Ghost, theme, 0.);
        let (_, lit) = colors(SwapVariant::Ghost, theme, 1.);
        assert_ne!(muted, lit);
    }
}

struct Page {
    label: &'static str,
    log: Rc<RefCell<Vec<&'static str>>>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let log = self.log.clone();
        div().p(px(20.)).flex().items_start().child(
            ActionSwapButton::new("swap", self.label).cap("⌘↵").debug_name("swap").on_click(move |_, _, _| log.borrow_mut().push("click")),
        )
    }
}

fn open(reduce: bool, cx: &mut TestAppContext) -> (Entity<Page>, &mut VisualTestContext, Rc<RefCell<Vec<&'static str>>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let l = log.clone();
    let (page, cx) = cx.add_window_view(move |_, _| Page { label: "Commit", log: l });
    for _ in 0..4 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    (page, cx, log)
}

#[gpui_kit::test]
fn the_button_is_28_tall_and_a_click_runs_it(cx: &mut TestAppContext) {
    let (_, cx, log) = open(true, cx);
    let b = cx.debug_bounds("swap").unwrap();
    assert_eq!(f32::from(b.size.height), HEIGHT);
    cx.simulate_click(b.center(), Modifiers::default());
    assert_eq!(*log.borrow(), vec!["click"]);
}

#[gpui_kit::test]
fn the_button_takes_the_width_of_the_new_words_at_once_and_the_old_ones_roll_out(cx: &mut TestAppContext) {
    motion::clock::freeze();
    let (page, cx, _) = open(false, cx);
    let commit = f32::from(cx.debug_bounds("swap").unwrap().size.width);
    page.update(cx, |p, cx| {
        p.label = "Open pull request";
        cx.notify();
    });
    for _ in 0..3 {
        motion::clock::advance(Duration::from_millis(20));
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    let after = f32::from(cx.debug_bounds("swap").unwrap().size.width);
    assert!(after > commit + 40., "wider for the longer words: {commit} then {after}");
}
