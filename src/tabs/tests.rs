use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Entity, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn the_glide_is_the_web_spring_and_settles_without_overshoot() {
    assert_eq!((GLIDE.stiffness, GLIDE.damping, GLIDE.mass), (245., 36., 1.2));
    let mut a = Animated::new(GLIDE, 0.);
    a.set_target(100.);
    let mut top: f32 = 0.;
    for _ in 0..600 {
        a.step(1. / 60., false);
        top = top.max(a.value());
    }
    assert!(top <= 100.5, "no overshoot: {top}");
    assert!(a.is_settled());
}

#[test]
fn the_looks_have_the_web_paddings_and_gaps() {
    assert_eq!((TabsVariant::Pill.pad(), TabsVariant::Pill.gap()), (4., 4.));
    assert_eq!((TabsVariant::Segment.pad(), TabsVariant::Segment.gap()), (2., 0.));
    assert_eq!(TabsVariant::Underline.tab_pad(), (12., 0.));
    assert_eq!(TabsVariant::Pill.tab_pad(), (14., 6.));
    assert_eq!(UNDERLINE_HEIGHT, 44.);
}

#[test]
fn the_share_of_a_tab_the_indicator_covers_follows_the_glide() {
    assert_eq!(covered(100., 50., 100., 50.), 1.);
    assert_eq!(covered(100., 50., 0., 50.), 0.);
    assert_eq!(covered(100., 50., 75., 50.), 0.5);
    assert_eq!(covered(100., 0., 100., 50.), 0.);
}

struct Page {
    selected: Option<usize>,
    variant: TabsVariant,
    log: Rc<RefCell<Vec<usize>>>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        let log = self.log.clone();
        div().p(px(20.)).flex().items_start().child(
            Tabs::new(
                "tabs",
                self.variant,
                [Tab::new("Overview").debug_name("t0"), Tab::new("Files").debug_name("t1"), Tab::new("Reading").pending(true).debug_name("t2")],
                self.selected,
            )
            .on_select(move |i, _, cx| {
                log.borrow_mut().push(i);
                this.update(cx, |p, cx| {
                    p.selected = Some(i);
                    cx.notify();
                });
            }),
        )
    }
}

fn open(variant: TabsVariant, cx: &mut TestAppContext) -> (Entity<Page>, &mut VisualTestContext, Rc<RefCell<Vec<usize>>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let l = log.clone();
    let (page, cx) = cx.add_window_view(move |_, _| Page { selected: Some(0), variant, log: l });
    for _ in 0..4 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    (page, cx, log)
}

fn centre(cx: &mut VisualTestContext, name: &'static str) -> gpui_kit::Point<Pixels> {
    cx.debug_bounds(name).unwrap_or_else(|| panic!("no {name}")).center()
}

#[gpui_kit::test]
fn a_click_chooses_a_tab_and_the_indicator_lies_under_it(cx: &mut TestAppContext) {
    let (page, cx, log) = open(TabsVariant::Pill, cx);
    let first = cx.debug_bounds("tabs-indicator").expect("the indicator is drawn");
    assert_eq!(first.left(), cx.debug_bounds("t0").unwrap().left());
    let at = centre(cx, "t1");
    cx.simulate_click(at, Modifiers::default());
    for _ in 0..4 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    assert_eq!(*log.borrow(), vec![1]);
    let now = cx.debug_bounds("tabs-indicator").unwrap();
    assert_eq!(now.left(), cx.debug_bounds("t1").unwrap().left(), "under Reduce Motion the indicator is there at once");
    assert_eq!(now.size.width, cx.debug_bounds("t1").unwrap().size.width);
}

#[gpui_kit::test]
fn at_zoom_the_indicator_still_lies_under_the_chosen_tab(cx: &mut TestAppContext) {
    crate::scale::set_zoom(1.5);
    let (page, cx, _) = open(TabsVariant::Segment, cx);
    let at = centre(cx, "t1");
    cx.simulate_click(at, Modifiers::default());
    for _ in 0..4 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    let now = cx.debug_bounds("tabs-indicator").unwrap();
    let tab = cx.debug_bounds("t1").unwrap();
    crate::scale::set_zoom(1.);
    assert!((f32::from(now.left()) - f32::from(tab.left())).abs() < 0.5, "left: {now:?} against {tab:?}");
    assert!((f32::from(now.size.width) - f32::from(tab.size.width)).abs() < 0.5, "width: {now:?} against {tab:?}");
}

#[gpui_kit::test]
fn a_pending_tab_cannot_be_chosen(cx: &mut TestAppContext) {
    let (_, cx, log) = open(TabsVariant::Pill, cx);
    let at = centre(cx, "t2");
    cx.simulate_click(at, Modifiers::default());
    cx.run_until_parked();
    assert!(log.borrow().is_empty());
}

#[gpui_kit::test]
fn an_underline_tab_is_44_tall_with_a_1px_line_under_the_chosen_one(cx: &mut TestAppContext) {
    let (_, cx, _) = open(TabsVariant::Underline, cx);
    assert_eq!(f32::from(cx.debug_bounds("t0").unwrap().size.height), UNDERLINE_HEIGHT);
    let line = cx.debug_bounds("tabs-indicator").unwrap();
    assert_eq!(f32::from(line.size.height), 1.);
    assert_eq!(line.bottom(), cx.debug_bounds("t0").unwrap().bottom());
}

#[gpui_kit::test]
fn the_keys_choose_the_focused_tab(cx: &mut TestAppContext) {
    let (_, cx, log) = open(TabsVariant::Segment, cx);
    cx.update(|window, cx| window.focus_next(cx));
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("enter");
    assert_eq!(*log.borrow(), vec![1], "the second tab stop");
}

/// The four editor-tab designs (for Alex to pick from): a 28px tab, no underline rule, and a marker that glides.
#[test]
fn the_editor_designs_are_28px_tabs_with_no_rule() {
    for variant in [TabsVariant::Chip, TabsVariant::ChipLine, TabsVariant::Dot, TabsVariant::Tick] {
        assert!(variant.is_editor());
        assert_eq!((variant.pad(), variant.tab_pad(), variant.gap()), (0., (10., 0.), 4.), "{variant:?}");
    }
    assert!(!TabsVariant::Underline.is_editor());
    assert_eq!(EDITOR_HEIGHT, 28.);
}
