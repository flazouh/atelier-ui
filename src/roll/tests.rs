use std::time::Duration;

use gpui_kit::{Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use super::*;
use crate::{
    motion,
    theme::{Appearance, set_appearance},
};

#[test]
fn icons_rise_from_80_percent_below_and_words_from_85() {
    assert_eq!(entering(Kind::Icon, 1., 0.), (0.8, 0.72));
    assert_eq!(entering(Kind::Icon, 0., 1.), (0., 1.));
    let (y, o) = entering(Kind::Words, 1., 0.);
    assert!((y - 0.85).abs() < 1e-6 && (o - 0.76).abs() < 1e-6);
}

#[test]
fn the_old_content_leaves_upward_to_half_opacity() {
    assert_eq!(leaving(Kind::Icon, 0.), (0., 1.));
    assert_eq!(leaving(Kind::Icon, 1.), (-0.8, 0.5));
    assert_eq!(leaving(Kind::Words, 1.), (-0.85, 0.5));
    assert_eq!((Kind::Icon.exit_seconds(), Kind::Words.exit_seconds()), (0.22, 0.2));
}

#[test]
fn the_rise_is_the_web_spring() {
    assert_eq!((RISE.stiffness, RISE.damping, RISE.mass), (210., 24., 0.85));
}

struct Page {
    key: &'static str,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        div().p(px(20.)).flex().child(Roll::new("roll", self.key, Kind::Words, px(16.), |key: &&'static str| {
            let name = format!("roll-{key}");
            div().h(px(16.)).debug_selector(move || name.clone()).child(*key).into_any_element()
        }))
    }
}

fn open(reduce: bool, cx: &mut TestAppContext) -> (Entity<Page>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
    });
    let (page, cx) = cx.add_window_view(|_, _| Page { key: "Working" });
    cx.run_until_parked();
    (page, cx)
}

fn frames(page: &Entity<Page>, cx: &mut VisualTestContext, n: usize) {
    for _ in 0..n {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
}

#[gpui_kit::test]
fn a_new_key_comes_up_from_below_and_the_old_one_is_gone_when_it_ends(cx: &mut TestAppContext) {
    motion::clock::freeze();
    let (page, cx) = open(false, cx);
    let rest = cx.debug_bounds("roll-Working").unwrap().origin.y;
    page.update(cx, |p, cx| {
        p.key = "Done";
        cx.notify();
    });
    motion::clock::advance(Duration::from_millis(60));
    frames(&page, cx, 2);
    let start = cx.debug_bounds("roll-Done");
    assert!(start.is_some_and(|b| b.origin.y > rest), "starts below its place");
    assert!(cx.debug_bounds("roll-Working").is_some_and(|b| b.origin.y < rest), "the old one goes up");
    for _ in 0..60 {
        motion::clock::advance(Duration::from_millis(30));
        frames(&page, cx, 1);
    }
    assert_eq!(cx.debug_bounds("roll-Done").unwrap().origin.y, rest, "at rest in its place");
    assert!(cx.debug_bounds("roll-Working").is_none(), "the old one is gone");
}

#[gpui_kit::test]
fn under_reduce_motion_the_content_changes_at_once(cx: &mut TestAppContext) {
    let (page, cx) = open(true, cx);
    let rest = cx.debug_bounds("roll-Working").unwrap().origin.y;
    page.update(cx, |p, cx| {
        p.key = "Done";
        cx.notify();
    });
    frames(&page, cx, 2);
    assert_eq!(cx.debug_bounds("roll-Done").unwrap().origin.y, rest);
    assert!(cx.debug_bounds("roll-Working").is_none());
}
