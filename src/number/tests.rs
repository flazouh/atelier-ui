use std::time::Duration;

use gpui_kit::{Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use super::*;
use crate::{
    motion,
    theme::{Appearance, set_appearance},
};

#[test]
fn text_is_cut_into_runs_of_digits_and_words() {
    assert_eq!(runs("+12"), vec![("+".into(), false), ("12".into(), true)]);
    assert_eq!(runs("3 of 12 reviewed"), vec![("3".into(), true), (" of ".into(), false), ("12".into(), true), (" reviewed".into(), false)]);
    assert_eq!(runs("none"), vec![("none".into(), false)]);
    assert!(runs("").is_empty());
}

struct Page {
    text: &'static str,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        div().p(px(20.)).flex().items_start().child(div().debug_selector(|| "digits".into()).child(Digits::new("d", self.text, px(20.))))
    }
}

fn open(reduce: bool, cx: &mut TestAppContext) -> (Entity<Page>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
    });
    let (page, cx) = cx.add_window_view(|_, _| Page { text: "+12" });
    for _ in 0..3 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    (page, cx)
}

#[gpui_kit::test]
fn each_digit_has_a_slot_06_em_wide_and_the_sign_is_plain(cx: &mut TestAppContext) {
    let (page, cx) = open(true, cx);
    let two = f32::from(cx.debug_bounds("digits").unwrap().size.width);
    page.update(cx, |p, cx| {
        p.text = "+123";
        cx.notify();
    });
    for _ in 0..3 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    let three = f32::from(cx.debug_bounds("digits").unwrap().size.width);
    assert!((three - two - 12.).abs() < 0.5, "one more digit is 0.6 * 20 = 12px wider: {two} then {three}");
}

#[gpui_kit::test]
fn a_changed_digit_rolls_and_settles_and_the_box_keeps_its_width(cx: &mut TestAppContext) {
    motion::clock::freeze();
    let (page, cx) = open(false, cx);
    let before = f32::from(cx.debug_bounds("digits").unwrap().size.width);
    page.update(cx, |p, cx| {
        p.text = "+19";
        cx.notify();
    });
    for _ in 0..30 {
        motion::clock::advance(Duration::from_millis(20));
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    assert_eq!(f32::from(cx.debug_bounds("digits").unwrap().size.width), before, "same slots");
}

#[test]
fn the_slot_is_the_line_height_of_the_text_around() {
    assert_eq!((line_for(11.), line_for(12.), line_for(14.), line_for(16.), line_for(20.)), (16., 16., 20., 24., 28.));
}
#[gpui_kit::test]
fn a_count_row_keeps_its_height_before_during_and_after_a_roll(cx: &mut TestAppContext) {
    motion::clock::freeze();
    let (page, cx) = open(false, cx);
    let height = |cx: &mut VisualTestContext| f32::from(cx.debug_bounds("digits").unwrap().size.height);
    let rest = height(cx);
    assert_eq!(rest, line_for(20.), "the line height of 20px text");
    page.update(cx, |p, cx| {
        p.text = "+19";
        cx.notify();
    });
    for step in 0..30 {
        motion::clock::advance(Duration::from_millis(10));
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
        assert_eq!(height(cx), rest, "step {step} of the roll");
    }
    assert_eq!(height(cx), rest);
}
