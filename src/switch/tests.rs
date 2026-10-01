use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Entity, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn the_track_is_48_by_28_and_the_thumb_travels_20() {
    assert_eq!((WIDTH, HEIGHT, PAD, THUMB, TRAVEL), (48., 28., 4., 20., 20.));
    assert_eq!((THUMB_SPRING.stiffness, THUMB_SPRING.damping, THUMB_SPRING.mass), (800., 80., 4.));
}

#[test]
fn the_thumb_overshoots_by_four_percent_at_most_and_settles() {
    let mut a = Animated::new(THUMB_SPRING, 0.);
    a.set_target(1.);
    let mut top: f32 = 0.;
    for _ in 0..600 {
        a.step(1. / 60., false);
        top = top.max(a.value());
    }
    assert!(top > 1.0 && top <= 1.05, "one small overshoot, damping ratio 0.707: {top}");
    assert!(a.is_settled());
}

#[test]
fn a_pressed_thumb_stretches_toward_the_side_it_came_from() {
    assert_eq!(thumb_span(0., 0., false), (4., 20.));
    assert_eq!(thumb_span(1., 0., true), (24., 20.));
    assert_eq!(thumb_span(0., 1., false), (4., 24.), "off: the stretch is to the right, toward where it goes");
    assert_eq!(thumb_span(1., 1., true), (20., 24.), "on: the stretch is to the left, and the right edge holds");
}

#[test]
fn a_disabled_thumb_shakes_after_200ms_and_settles_at_800() {
    assert_eq!(shake_offset(0.1), 0.);
    assert!(shake_offset(0.2 + 0.15).abs() > 0.5 || shake_offset(0.2 + 0.3).abs() > 0.5);
    assert!(shake_offset(0.8).abs() < 1e-3);
}

#[test]
fn the_track_goes_from_the_muted_ink_to_the_primary() {
    for theme in crate::themes::all() {
        let off = track_fill(theme, theme.background, 0.);
        let on = track_fill(theme, theme.background, 1.);
        assert_ne!(off, on, "{}", theme.name);
    }
}

struct Page {
    on: bool,
    disabled: bool,
    log: Rc<RefCell<Vec<bool>>>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let (this, log) = (cx.entity(), self.log.clone());
        div().p(px(20.)).flex().items_start().child(
            Switch::new("sw", self.on).label("Group by project").cap("⌘⇧G").disabled(self.disabled).debug_name("sw").on_change(move |value, _, cx| {
                log.borrow_mut().push(value);
                this.update(cx, |p, cx| {
                    p.on = value;
                    cx.notify();
                });
            }),
        )
    }
}

fn open(on: bool, disabled: bool, cx: &mut TestAppContext) -> (Entity<Page>, &mut VisualTestContext, Rc<RefCell<Vec<bool>>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let l = log.clone();
    let (page, cx) = cx.add_window_view(move |_, _| Page { on, disabled, log: l });
    for _ in 0..4 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    (page, cx, log)
}

#[gpui_kit::test]
fn the_track_is_48_by_28_and_the_thumb_sits_at_the_end_the_state_says(cx: &mut TestAppContext) {
    let (page, cx, _) = open(false, false, cx);
    let track = cx.debug_bounds("sw").unwrap();
    assert_eq!((f32::from(track.size.width), f32::from(track.size.height)), (WIDTH, HEIGHT));
    let thumb = cx.debug_bounds("sw-thumb").unwrap();
    assert_eq!(f32::from(thumb.left() - track.left()), PAD);
    page.update(cx, |p, cx| {
        p.on = true;
        cx.notify();
    });
    for _ in 0..3 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    let thumb = cx.debug_bounds("sw-thumb").unwrap();
    assert_eq!(f32::from(track.right() - thumb.right()), PAD);
}

#[gpui_kit::test]
fn a_click_on_the_track_or_the_label_toggles_and_the_cap_shows_once_after_the_label(cx: &mut TestAppContext) {
    let (_, cx, log) = open(false, false, cx);
    let at = cx.debug_bounds("sw").unwrap().center();
    cx.simulate_click(at, Modifiers::default());
    assert_eq!(*log.borrow(), vec![true]);
    let at = cx.debug_bounds("sw").unwrap().center();
    cx.simulate_click(at, Modifiers::default());
    assert_eq!(*log.borrow(), vec![true, false]);
}

#[gpui_kit::test]
fn a_disabled_switch_does_not_toggle(cx: &mut TestAppContext) {
    let (_, cx, log) = open(false, true, cx);
    let at = cx.debug_bounds("sw").unwrap().center();
    cx.simulate_click(at, Modifiers::default());
    assert!(log.borrow().is_empty());
}

#[gpui_kit::test]
fn enter_and_space_toggle_the_focused_switch(cx: &mut TestAppContext) {
    let (_, cx, log) = open(false, false, cx);
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("space");
    cx.simulate_keystrokes("enter");
    assert_eq!(*log.borrow(), vec![true, false]);
}

#[test]
fn a_compact_switch_is_a_32_by_20_track_with_the_same_motion() {
    let d = Dims::COMPACT;
    assert_eq!((d.width, d.height, d.thumb, d.travel()), (32., 20., 14., 12.));
    assert_eq!(span(d, 1., 0., true), (3. + 12., 14.));
    assert_eq!(span(Dims::STANDARD, 0., 0., false), thumb_span(0., 0., false));
}
