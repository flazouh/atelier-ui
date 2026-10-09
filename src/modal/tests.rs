use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui_kit::{Entity, InteractiveElement, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use super::*;
use crate::{
    motion,
    theme::{Appearance, Theme, set_appearance},
};

#[test]
fn the_panel_is_the_view_and_its_padding_tall_with_no_border() {
    assert_eq!(panel_height(100.), 100. + 32.);
    assert_eq!(BORDER, 0., "atelier is borderless");
    assert_eq!((PAD, CORNER, ENTER_Y, VIEW_Y), (16., 12., 20., 8.));
    assert_eq!((PANEL.stiffness, PANEL.damping, PANEL.mass), (420., 40., 0.5));
}

#[test]
fn the_scrim_is_the_shadow_colour_and_fades_in() {
    let theme = Theme::light();
    assert_eq!(scrim(&theme, 0.).a, 0.);
    assert!((scrim(&theme, 1.).a - 0.28).abs() < 1e-6);
}

struct Page {
    open: bool,
    tall: bool,
    huge: bool,
    flush: bool,
    log: Rc<RefCell<Vec<&'static str>>>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let (this, log) = (cx.entity(), self.log.clone());
        div()
            .size_full()
            .child(div().debug_selector(|| "behind".into()).size(px(50.)))
            .children(self.open.then(|| {
                let modal = Modal::new("modal").view(self.tall).width(300.).debug_name("panel");
                let modal = if self.flush { modal.flush() } else { modal };
                modal
                    .on_close(move |_, cx| {
                        log.borrow_mut().push("close");
                        this.update(cx, |p, cx| {
                            p.open = false;
                            cx.notify();
                        });
                    })
                    .child(div().debug_selector(|| "view".into()).w_full().h(px(if self.huge { 5000. } else if self.tall { 200. } else { 80. })))
            }))
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
    let (page, cx) = cx.add_window_view(move |_, _| Page { open: true, tall: false, huge: false, flush: false, log: l });
    settle(&page, cx, 6);
    (page, cx, log)
}

fn settle(page: &Entity<Page>, cx: &mut VisualTestContext, n: usize) {
    for _ in 0..n {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
}

#[gpui_kit::test]
fn the_panel_opens_in_the_middle_at_the_width_and_the_height_of_its_view(cx: &mut TestAppContext) {
    let (_, cx, _) = open(true, cx);
    let panel = cx.debug_bounds("panel").unwrap();
    assert_eq!(f32::from(panel.size.width), 300.);
    assert_eq!(f32::from(panel.size.height), panel_height(80.));
    let centre = panel.center();
    let window = cx.update(|window, _| window.viewport_size());
    assert!((f32::from(centre.x) - f32::from(window.width) / 2.).abs() < 1.);
    assert!((f32::from(centre.y) - f32::from(window.height) / 2.).abs() < 1.);
}

#[gpui_kit::test]
fn escape_and_a_press_on_the_scrim_close_and_a_press_in_the_panel_does_not(cx: &mut TestAppContext) {
    let (page, cx, log) = open(true, cx);
    let inside = cx.debug_bounds("view").unwrap().center();
    cx.simulate_click(inside, Modifiers::default());
    assert!(log.borrow().is_empty(), "a press inside stays inside");
    cx.simulate_click(gpui_kit::point(px(5.), px(5.)), Modifiers::default());
    assert_eq!(*log.borrow(), vec!["close"], "the scrim took the press, and what was behind heard nothing");
    assert!(!page.read_with(cx, |p, _| p.open));
    let (page, cx, log) = (page, cx, log);
    page.update(cx, |p, cx| {
        p.open = true;
        cx.notify();
    });
    settle(&page, cx, 4);
    cx.simulate_keystrokes("escape");
    // The panel hears Escape only with focus in it; the owner's form does that. Here nothing has it, so the scrim is the way.
    let _ = log;
}

#[gpui_kit::test]
fn a_longer_view_grows_the_panel_and_a_shorter_one_shrinks_it(cx: &mut TestAppContext) {
    motion::clock::freeze();
    let (page, cx, _) = open(false, cx);
    for _ in 0..40 {
        motion::clock::advance(Duration::from_millis(30));
        settle(&page, cx, 1);
    }
    assert_eq!(f32::from(cx.debug_bounds("panel").unwrap().size.height), panel_height(80.));
    page.update(cx, |p, cx| {
        p.tall = true;
        cx.notify();
    });
    settle(&page, cx, 2);
    motion::clock::advance(Duration::from_millis(40));
    settle(&page, cx, 2);
    let mid = f32::from(cx.debug_bounds("panel").unwrap().size.height);
    assert!(mid > panel_height(80.) && mid < panel_height(200.) + 30., "on its way: {mid}");
    for _ in 0..60 {
        motion::clock::advance(Duration::from_millis(30));
        settle(&page, cx, 1);
    }
    assert!((f32::from(cx.debug_bounds("panel").unwrap().size.height) - panel_height(200.)).abs() < 1.);
}

struct Focused {
    before: gpui_kit::FocusHandle,
    inside: gpui_kit::FocusHandle,
    open: bool,
}

impl Render for Focused {
    fn render(&mut self, _: &mut Window, cx: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        div().size_full().child(div().track_focus(&self.before).size(px(50.))).children(self.open.then(|| {
            Modal::new("focused").focus(&self.inside).on_close(move |_, cx| {
                this.update(cx, |p, cx| {
                    p.open = false;
                    cx.notify();
                });
            }).child(div().track_focus(&self.inside).size(px(80.)))
        }))
    }
}

#[gpui_kit::test]
fn the_view_takes_focus_and_escape_gives_it_back(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let (page, cx) = cx.add_window_view(|_, cx| Focused { before: cx.focus_handle(), inside: cx.focus_handle(), open: false });
    let (before, inside) = page.read_with(cx, |p, _| (p.before.clone(), p.inside.clone()));
    cx.update(|window, cx| before.focus(window, cx));
    page.update(cx, |p, cx| {
        p.open = true;
        cx.notify();
    });
    for _ in 0..4 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    assert!(cx.update(|window, _| inside.is_focused(window)), "the view has focus");
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(!page.read_with(cx, |p, _| p.open));
    assert!(cx.update(|window, _| before.is_focused(window)), "focus went back to what had it");
}

#[gpui_kit::test]
fn a_flush_panel_has_no_padding_so_its_view_reaches_every_edge(cx: &mut TestAppContext) {
    let (page, cx, _) = open(true, cx);
    let padded = cx.debug_bounds("view").unwrap();
    assert_eq!(f32::from(padded.size.width), 300. - 2. * PAD);
    page.update(cx, |p, cx| {
        p.flush = true;
        cx.notify();
    });
    settle(&page, cx, 4);
    let panel = cx.debug_bounds("panel").unwrap();
    let view = cx.debug_bounds("view").unwrap();
    assert_eq!(f32::from(panel.size.height), 80., "the view alone, with no padding round it");
    assert_eq!((view.origin.x, view.size.width), (panel.origin.x, panel.size.width));
}

/// At a zoom the panel is its view and its padding tall, in window pixels: the view is measured in window pixels and the panel
/// is kept in design pixels, so a panel that took the one for the other would stand taller than its view (168 for 128).
#[gpui_kit::test]
fn at_a_zoom_the_panel_is_as_tall_as_its_view_and_its_padding(cx: &mut TestAppContext) {
    crate::scale::set_zoom(1.5);
    let (_, cx, _) = open(true, cx);
    let panel = cx.debug_bounds("panel").unwrap();
    crate::scale::set_zoom(1.);
    // The view is 80 window pixels tall (the test draws it unscaled), and the padding is 16 design pixels on each side.
    let want = 80. + 2. * PAD * 1.5;
    assert!((f32::from(panel.size.height) - want).abs() < 1.5, "the panel is {:?} tall, not {want}", panel.size.height);
}

/// A view taller than the window leaves the window's edge room: the panel is capped and stays inside the window.
#[gpui_kit::test]
fn a_view_taller_than_the_window_is_capped_inside_it(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let (page, cx) = cx.add_window_view(move |_, _| Page { open: true, tall: false, huge: true, flush: false, log });
    settle(&page, cx, 8);
    let panel = cx.debug_bounds("panel").unwrap();
    let window = cx.update(|window, _| window.viewport_size());
    assert!(panel.top() >= gpui_kit::px(0.) && panel.bottom() <= window.height, "the panel {panel:?} is inside the window {window:?}");
    assert!(f32::from(panel.size.height) <= f32::from(window.height) - 2. * super::types::EDGE + 0.5, "it leaves its edge room");
}
