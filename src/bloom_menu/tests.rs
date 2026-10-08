use std::{cell::RefCell, rc::Rc};

use gpui_kit::{AppContext as _, Entity, Modifiers, TestAppContext, VisualTestContext, point, size};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn six_choices_are_two_rows_of_three_in_a_panel_420_wide_and_240_tall() {
    assert_eq!(rows(6), 2);
    assert_eq!(rows(7), 3);
    assert_eq!(rows(0), 1);
    assert_eq!(panel_size(6), (420., 45. + 192. + 1. + 2.), "the header, two rows of 96 with a line between, and the border");
}

#[test]
fn the_four_corners_are_equally_far_from_the_middle_so_they_arrive_together() {
    let d = |i| distance(i, 6);
    assert_eq!((d(0), d(2), d(3), d(5)), (d(0), d(0), d(0), d(0)));
    assert!((d(0) - (1f32 + 0.25).sqrt()).abs() < 1e-5, "one column and half a row from the centre");
    assert!((d(1) - 0.5).abs() < 1e-5 && (d(4) - 0.5).abs() < 1e-5, "the middle column is half a row off");
    assert!((delay(1, 6) - (0.1 + 0.5 * 0.07)).abs() < 1e-5);
    assert!(delay(0, 6) > delay(1, 6), "the corners come after the middle");
}

#[test]
fn the_box_is_the_button_at_rest_and_the_panel_when_open_and_grows_from_its_middle() {
    assert_eq!(box_size(0., 6), (144., 44.));
    assert_eq!(box_size(1., 6), (420., 240.));
    let (w, h) = box_size(0.5, 6);
    assert!((w - 282.).abs() < 1e-3 && (h - 142.).abs() < 1e-3);
}

#[test]
fn the_iris_starts_45_and_34_percent_in_and_is_fully_open_at_the_end() {
    assert_eq!(iris_cut(0.), (0.45, 0.34));
    assert_eq!(iris_cut(1.), (0., 0.));
    let (v, h) = iris_cut(0.5);
    assert!((v - 0.225).abs() < 1e-6 && (h - 0.17).abs() < 1e-6);
}

/// The menu a little way in from the window's corner, as it is on a page: its open panel reaches 138 px to the left and 98 up.
struct Page {
    menu: Entity<BloomMenu>,
}

impl gpui_kit::Render for Page {
    fn render(&mut self, _: &mut gpui_kit::Window, _: &mut gpui_kit::Context<Self>) -> impl gpui_kit::IntoElement {
        use gpui_kit::{ParentElement, Styled};
        gpui_kit::div().size_full().child(gpui_kit::div().ml(gpui_kit::px(300.)).mt(gpui_kit::px(200.)).child(self.menu.clone()))
    }
}

fn open(reduce: bool, cx: &mut TestAppContext) -> (Entity<BloomMenu>, Rc<RefCell<Vec<String>>>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
        crate::motion::clock::freeze();
    });
    let (page, cx) = cx.add_window_view(|_, cx| Page { menu: cx.new(|cx| BloomMenu::new("bloom", default_items(), cx)) });
    let menu = page.read_with(cx, |p, _| p.menu.clone());
    cx.simulate_resize(size(gpui_kit::px(800.), gpui_kit::px(600.)));
    let heard = Rc::new(RefCell::new(Vec::new()));
    let log = heard.clone();
    let sub = cx.update(|_, cx| {
        cx.subscribe(&menu, move |_, event: &BloomEvent, _| {
            log.borrow_mut().push(match event {
                BloomEvent::Select(label) => format!("select {label}"),
                BloomEvent::Toggled(open) => format!("toggled {open}"),
            })
        })
    });
    std::mem::forget(sub);
    settle(&menu, cx);
    (menu, heard, cx)
}

fn settle(menu: &Entity<BloomMenu>, cx: &mut VisualTestContext) {
    for _ in 0..4 {
        cx.run_until_parked();
        menu.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
}

fn bounds(cx: &mut VisualTestContext) -> gpui_kit::Bounds<gpui_kit::Pixels> {
    cx.debug_bounds("bloom-box").expect("the box is drawn")
}

fn center_of(b: gpui_kit::Bounds<gpui_kit::Pixels>) -> (f32, f32) {
    (f32::from(b.origin.x + b.size.width / 2.), f32::from(b.origin.y + b.size.height / 2.))
}

#[gpui_kit::test]
fn a_press_opens_the_button_into_the_panel_about_its_own_middle(cx: &mut TestAppContext) {
    let (menu, heard, cx) = open(true, cx);
    let button = bounds(cx);
    assert_eq!((f32::from(button.size.width), f32::from(button.size.height)), (144., 44.));
    let at = button.center();
    cx.simulate_click(at, Modifiers::default());
    settle(&menu, cx);
    assert!(menu.read_with(cx, |m, _| m.is_open()));
    let panel = bounds(cx);
    assert_eq!((f32::from(panel.size.width), f32::from(panel.size.height)), (420., 240.), "the reduced-motion end state");
    let (a, b) = (center_of(button), center_of(panel));
    assert!((a.0 - b.0).abs() < 0.6 && (a.1 - b.1).abs() < 0.6, "one middle: {a:?} {b:?}");
    assert_eq!(*heard.borrow(), ["toggled true"]);
}

#[gpui_kit::test]
fn escape_the_cross_a_press_outside_and_a_choice_each_shut_it_and_only_a_choice_selects(cx: &mut TestAppContext) {
    let (menu, heard, cx) = open(true, cx);
    let shut = |menu: &Entity<BloomMenu>, cx: &mut VisualTestContext| menu.read_with(cx, |m, _| !m.is_open());
    let open_it = |menu: &Entity<BloomMenu>, cx: &mut VisualTestContext| {
        let at = bounds(cx).center();
        cx.simulate_click(at, Modifiers::default());
        settle(menu, cx);
        assert!(menu.read_with(cx, |m, _| m.is_open()));
    };
    open_it(&menu, cx);
    cx.simulate_keystrokes("escape");
    settle(&menu, cx);
    assert!(shut(&menu, cx), "Escape");
    open_it(&menu, cx);
    let at = cx.debug_bounds("bloom-close").expect("the cross").center();
    cx.simulate_click(at, Modifiers::default());
    settle(&menu, cx);
    assert!(shut(&menu, cx), "the cross");
    open_it(&menu, cx);
    cx.simulate_click(point(gpui_kit::px(20.), gpui_kit::px(580.)), Modifiers::default());
    settle(&menu, cx);
    assert!(shut(&menu, cx), "a press outside");
    assert!(heard.borrow().iter().all(|e| !e.starts_with("select")), "none of those chose anything");
    open_it(&menu, cx);
    let at = cx.debug_bounds("bloom-cell-4").expect("a cell").center();
    cx.simulate_click(at, Modifiers::default());
    settle(&menu, cx);
    assert!(shut(&menu, cx), "a choice");
    assert_eq!(heard.borrow().iter().filter(|e| e.starts_with("select")).collect::<Vec<_>>(), ["select Reminder"]);
}

#[gpui_kit::test]
fn with_motion_the_box_grows_on_the_folder_spring_and_the_words_and_choices_follow(cx: &mut TestAppContext) {
    let (menu, _, cx) = open(false, cx);
    let at = bounds(cx).center();
    cx.simulate_click(at, Modifiers::default());
    let (m, words, iris, first) = menu.read_with(cx, |m, _| (m.morph.value(), m.words.value(), m.iris.value(), m.arrive[0].value()));
    assert!(m < 0.05 && words < 0.05 && iris < 0.05 && first < 0.05, "at the first frame nothing has moved");
    crate::motion::clock::advance(std::time::Duration::from_millis(100));
    let (m, words, iris, corner, middle) = menu.read_with(cx, |m, _| (m.morph.value(), m.words.value(), m.iris.value(), m.arrive[0].value(), m.arrive[1].value()));
    assert!(m > 0.1, "the box is growing: {m}");
    assert!(words < 0.05, "the words wait 120 ms: {words}");
    assert!(iris > 0., "the iris started at 80 ms");
    assert!(corner < 0.05, "the corners wait 100 ms plus 78 ms: {corner}");
    assert!(middle < 0.05, "the middle waits 100 ms plus 35 ms: {middle}");
    crate::motion::clock::advance(std::time::Duration::from_millis(1500));
    let (m, words, iris, corner) = menu.read_with(cx, |m, _| (m.morph.value(), m.words.value(), m.iris.value(), m.arrive[0].value()));
    assert!((m - 1.).abs() < 0.01 && words == 1. && iris == 1. && (corner - 1.).abs() < 0.01);
    settle(&menu, cx);
    // Closing: the box goes back the way it came.
    cx.simulate_keystrokes("escape");
    crate::motion::clock::advance(std::time::Duration::from_millis(60));
    settle(&menu, cx);
    let b = bounds(cx);
    assert!(f32::from(b.size.width) > 144. && f32::from(b.size.width) < 420., "part way back: {:?}", b.size);
    crate::motion::clock::advance(std::time::Duration::from_millis(1500));
    settle(&menu, cx);
    assert_eq!(f32::from(bounds(cx).size.width).round(), 144.);
}
