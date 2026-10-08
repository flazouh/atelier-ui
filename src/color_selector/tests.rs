use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Context, Entity, IntoElement, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, hsla, px, size};

use super::*;
use crate::theme::{Appearance, set_appearance};

fn swatches(n: usize) -> Vec<Swatch> {
    (0..n).map(|i| Swatch::new(format!("s{i}"), hsla(i as f32 / 10., 0.7, 0.5, 1.), format!("Swatch {i}"))).collect()
}

#[test]
fn the_arrow_keys_wrap_and_pass_over_a_swatch_that_cannot_be_chosen() {
    let mut list = swatches(4);
    list[2] = list[2].clone().disabled(true);
    assert_eq!(step(&list, 1, true), Some(3), "over the disabled one");
    assert_eq!(step(&list, 3, true), Some(0), "the last wraps to the first");
    assert_eq!(step(&list, 0, false), Some(3), "the first wraps to the last");
    assert_eq!(step(&list, 3, false), Some(1));
    assert_eq!(step(&swatches(1), 0, true), None, "nothing else to reach");
    let all_off: Vec<Swatch> = swatches(3).into_iter().map(|s| s.disabled(true)).collect();
    assert_eq!(step(&all_off, 0, true), None);
}

#[test]
fn tab_reaches_the_chosen_swatch_or_else_the_first_one_that_can_be_chosen() {
    let mut list = swatches(3);
    assert_eq!(tab_stop(&list, Some(&"s2".into())), Some(2));
    assert_eq!(tab_stop(&list, None), Some(0));
    assert_eq!(tab_stop(&list, Some(&"nope".into())), Some(0));
    list[0] = list[0].clone().disabled(true);
    assert_eq!(tab_stop(&list, None), Some(1), "the first that can be chosen");
    list[2] = list[2].clone().disabled(true);
    assert_eq!(tab_stop(&list, Some(&"s2".into())), Some(1), "a chosen one that cannot be chosen is not a stop");
}

#[test]
fn a_press_sinks_the_disc_and_the_dot_and_the_ring_by_the_same_scale() {
    let (disc, dot, ring) = sizes(PRESS_SCALE);
    assert!((disc - 28. * 0.94).abs() < 1e-4 && (dot - 14. * 0.94).abs() < 1e-4 && (ring - 32. * 0.94).abs() < 1e-4);
    assert_eq!(sizes(1.), (28., 14., 32.), "a 28px disc, a 14px dot, and the ring 2px outside the disc");
}

/// The group as an owner keeps it: the value in the owner, the changes logged.
struct Owner {
    swatches: Vec<Swatch>,
    value: Option<SharedString>,
    disabled: bool,
    log: Rc<RefCell<Vec<String>>>,
}

impl Render for Owner {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (this, log) = (cx.entity().downgrade(), self.log.clone());
        div().w(px(600.)).p(px(20.)).child(
            ColorSelector::new("accent", self.swatches.clone())
                .label("Accent")
                .value(self.value.clone())
                .disabled(self.disabled)
                .on_change(move |v, _, cx| {
                    log.borrow_mut().push(v.to_string());
                    let v = v.clone();
                    this.update(cx, |o, cx| {
                        o.value = Some(v);
                        cx.notify();
                    })
                    .ok();
                }),
        )
    }
}

fn open<'a>(swatches: Vec<Swatch>, value: Option<&str>, disabled: bool, reduce: bool, cx: &'a mut TestAppContext) -> (Entity<Owner>, &'a mut VisualTestContext, Rc<RefCell<Vec<String>>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::motion::clock::freeze();
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let value = value.map(|v| SharedString::from(v.to_string()));
    let (owner, cx) = cx.add_window_view(move |_, _| Owner { swatches, value, disabled, log: seen });
    cx.simulate_resize(size(px(700.), px(300.)));
    for _ in 0..3 {
        cx.run_until_parked();
        owner.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
    (owner, cx, log)
}

fn frames(owner: &Entity<Owner>, cx: &mut VisualTestContext, n: usize) {
    for _ in 0..n {
        cx.run_until_parked();
        owner.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
}

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let at = cx.debug_bounds(selector).unwrap_or_else(|| panic!("{selector} is not drawn")).center();
    cx.simulate_click(at, gpui_kit::Modifiers::default());
    cx.run_until_parked();
}

#[gpui_kit::test]
fn a_click_chooses_a_swatch_and_choosing_the_chosen_one_says_nothing(cx: &mut TestAppContext) {
    let (owner, cx, log) = open(swatches(4), Some("s0"), false, true, cx);
    click(cx, "color-swatch-2");
    assert_eq!(*log.borrow(), ["s2"]);
    assert_eq!(owner.read_with(cx, |o, _| o.value.clone()), Some("s2".into()));
    click(cx, "color-swatch-2");
    assert_eq!(*log.borrow(), ["s2"], "the same swatch again is not a change");
}

#[gpui_kit::test]
fn a_swatch_that_cannot_be_chosen_and_a_disabled_group_ignore_clicks(cx: &mut TestAppContext) {
    let mut list = swatches(3);
    list[1] = list[1].clone().disabled(true);
    let (_, cx, log) = open(list, Some("s0"), false, true, cx);
    click(cx, "color-swatch-1");
    assert!(log.borrow().is_empty());
    let (_, cx, log) = open(swatches(3), Some("s0"), true, true, cx);
    click(cx, "color-swatch-2");
    assert!(log.borrow().is_empty());
}

#[gpui_kit::test]
fn the_arrow_keys_move_the_choice_and_the_outline_shows_on_the_focused_swatch(cx: &mut TestAppContext) {
    let mut list = swatches(4);
    list[1] = list[1].clone().disabled(true);
    let (owner, cx, log) = open(list, Some("s0"), false, true, cx);
    click(cx, "color-swatch-0");
    assert!(cx.debug_bounds("color-outline").is_none(), "a click gives no outline: it is for the keyboard");
    cx.simulate_keystrokes("right");
    assert_eq!(*log.borrow(), ["s2"], "over the disabled swatch");
    frames(&owner, cx, 3);
    assert!(cx.debug_bounds("color-outline").is_some(), "and the swatch it reached has the outline");
    cx.simulate_keystrokes("left");
    assert_eq!(*log.borrow(), ["s2", "s0"]);
    cx.simulate_keystrokes("left");
    assert_eq!(*log.borrow(), ["s2", "s0", "s3"], "the first wraps to the last");
}

#[gpui_kit::test]
fn under_reduce_motion_the_ring_is_on_the_new_swatch_at_once(cx: &mut TestAppContext) {
    let (owner, cx, _) = open(swatches(5), Some("s0"), false, true, cx);
    click(cx, "color-swatch-3");
    frames(&owner, cx, 3);
    let ring = cx.debug_bounds("color-ring").expect("the ring is drawn");
    let disc = cx.debug_bounds("color-swatch-3").unwrap();
    let off = |a: gpui_kit::Pixels, b: gpui_kit::Pixels| (f32::from(a) - f32::from(b)).abs();
    assert!(off(ring.center().x, disc.center().x) < 0.6 && off(ring.center().y, disc.center().y) < 0.6, "{ring:?} on {disc:?}");
    assert!((f32::from(ring.size.width) - 32.).abs() < 0.6, "2px outside a 28px disc");
}

#[gpui_kit::test]
fn with_motion_the_ring_glides_and_arrives(cx: &mut TestAppContext) {
    let (owner, cx, _) = open(swatches(5), Some("s0"), false, false, cx);
    let start = f32::from(cx.debug_bounds("color-ring").unwrap().center().x);
    click(cx, "color-swatch-3");
    frames(&owner, cx, 1);
    let target = f32::from(cx.debug_bounds("color-swatch-3").unwrap().center().x);
    let early = f32::from(cx.debug_bounds("color-ring").unwrap().center().x);
    assert!(early < target - 10., "the ring has not arrived a frame after the press: {early} of {target}");
    assert!(early >= start, "and it is on its way from {start}");
    crate::motion::clock::advance(std::time::Duration::from_millis(1200));
    frames(&owner, cx, 3);
    let late = f32::from(cx.debug_bounds("color-ring").unwrap().center().x);
    assert!((late - target).abs() < 0.6, "it settles on the swatch: {late} of {target}");
}

#[gpui_kit::test]
fn a_press_sinks_the_disc_and_letting_go_brings_it_back(cx: &mut TestAppContext) {
    let (owner, cx, _) = open(swatches(3), Some("s0"), false, false, cx);
    let at = cx.debug_bounds("color-swatch-1").unwrap().center();
    let rest = f32::from(cx.debug_bounds("color-disc-1").unwrap().size.width);
    cx.simulate_event(gpui_kit::MouseDownEvent { position: at, modifiers: Default::default(), button: gpui_kit::MouseButton::Left, click_count: 1, first_mouse: false });
    crate::motion::clock::advance(std::time::Duration::from_millis(300));
    frames(&owner, cx, 3);
    let held = f32::from(cx.debug_bounds("color-disc-1").unwrap().size.width);
    assert!(held < rest - 1.0, "held: {held}, at rest: {rest}");
    cx.simulate_event(gpui_kit::MouseUpEvent { position: at, modifiers: Default::default(), button: gpui_kit::MouseButton::Left, click_count: 1 });
    crate::motion::clock::advance(std::time::Duration::from_millis(600));
    frames(&owner, cx, 3);
    let back = f32::from(cx.debug_bounds("color-disc-1").unwrap().size.width);
    assert!((back - rest).abs() < 0.6, "back: {back}");
}

#[gpui_kit::test]
fn under_reduce_motion_a_press_does_not_sink(cx: &mut TestAppContext) {
    let (owner, cx, _) = open(swatches(3), Some("s0"), false, true, cx);
    let at = cx.debug_bounds("color-swatch-1").unwrap().center();
    let rest = f32::from(cx.debug_bounds("color-disc-1").unwrap().size.width);
    cx.simulate_event(gpui_kit::MouseDownEvent { position: at, modifiers: Default::default(), button: gpui_kit::MouseButton::Left, click_count: 1, first_mouse: false });
    frames(&owner, cx, 3);
    assert_eq!(f32::from(cx.debug_bounds("color-disc-1").unwrap().size.width), rest);
}

#[gpui_kit::test]
fn zoomed_in_the_ring_sits_on_the_chosen_swatch(cx: &mut TestAppContext) {
    crate::scale::set_zoom(1.5);
    let (owner, cx, _) = open(swatches(5), Some("s3"), false, true, cx);
    frames(&owner, cx, 3);
    let ring = cx.debug_bounds("color-ring").expect("the ring is drawn");
    let disc = cx.debug_bounds("color-swatch-3").unwrap();
    crate::scale::set_zoom(1.);
    let off = |a: gpui_kit::Pixels, b: gpui_kit::Pixels| (f32::from(a) - f32::from(b)).abs();
    assert!(
        off(ring.center().x, disc.center().x) < 0.6 && off(ring.center().y, disc.center().y) < 0.6,
        "{ring:?} on {disc:?}"
    );
}
