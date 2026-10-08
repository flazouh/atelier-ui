use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    Context, Entity, IntoElement, ParentElement, Render, Styled, TestAppContext, VisualTestContext,
    Window, div, px, size,
};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn a_partial_choice_shows_a_dash_whether_or_not_it_is_checked() {
    assert_eq!(Mark::of(false, false), Mark::None);
    assert_eq!(Mark::of(true, false), Mark::Tick);
    assert_eq!(Mark::of(false, true), Mark::Dash);
    assert_eq!(Mark::of(true, true), Mark::Dash);
}

#[test]
fn the_stroke_draws_itself_by_length_along_the_path() {
    assert_eq!(
        prefix(&TICK, 0.),
        vec![(5., 13.), (5., 13.)],
        "nothing drawn yet: a point"
    );
    assert_eq!(prefix(&TICK, 1.), TICK.to_vec());
    let half = prefix(&DASH, 0.5);
    assert_eq!(half, vec![(6., 12.), (12., 12.)], "half of the dash");
    // The tick's first leg is 4*sqrt(2) long and its second 10*sqrt(2): a quarter of the whole stops inside the first.
    let quarter = prefix(&TICK, 0.25);
    assert_eq!(quarter.len(), 2);
    assert!(
        quarter[1].0 > 5. && quarter[1].0 < 9.,
        "on the first leg: {quarter:?}"
    );
    assert_eq!(
        prefix(&TICK, 0.5).len(),
        3,
        "past the joint: start, joint, and a point on the second leg"
    );
    assert_eq!(prefix(&TICK, 2.), TICK.to_vec(), "past the end is the end");
    assert!(prefix(&[], 0.5).is_empty());
}

struct Owner {
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    log: Rc<RefCell<Vec<bool>>>,
}

impl Render for Owner {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (this, log) = (cx.entity().downgrade(), self.log.clone());
        div().p(px(20.)).child(
            Checkbox::new("cb", self.checked)
                .indeterminate(self.indeterminate)
                .disabled(self.disabled)
                .label("Accept")
                .on_change(move |v, _, cx| {
                    log.borrow_mut().push(v);
                    this.update(cx, |o, cx| {
                        o.checked = v;
                        cx.notify();
                    })
                    .ok();
                }),
        )
    }
}

fn open(
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    reduce: bool,
    cx: &mut TestAppContext,
) -> (
    Entity<Owner>,
    &mut VisualTestContext,
    Rc<RefCell<Vec<bool>>>,
) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::motion::clock::freeze();
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (owner, cx) = cx.add_window_view(move |_, _| Owner {
        checked,
        indeterminate,
        disabled,
        log: seen,
    });
    cx.simulate_resize(size(px(400.), px(200.)));
    frames(&owner, cx, 3);
    (owner, cx, log)
}

fn frames(owner: &Entity<Owner>, cx: &mut VisualTestContext, n: usize) {
    for _ in 0..n {
        cx.run_until_parked();
        owner.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
}

fn mark_width(cx: &mut VisualTestContext) -> Option<f32> {
    cx.debug_bounds("checkbox-mark")
        .map(|b| f32::from(b.size.width))
}

fn click(cx: &mut VisualTestContext) {
    let at = cx.debug_bounds("checkbox-box").unwrap().center();
    cx.simulate_click(at, gpui_kit::Modifiers::default());
    cx.run_until_parked();
}

#[gpui_kit::test]
fn a_click_asks_for_the_opposite_and_a_disabled_box_ignores_it(cx: &mut TestAppContext) {
    let (owner, cx, log) = open(false, false, false, true, cx);
    click(cx);
    frames(&owner, cx, 2);
    assert_eq!(*log.borrow(), [true]);
    click(cx);
    assert_eq!(*log.borrow(), [true, false]);
    let (_, cx, log) = open(true, false, true, true, cx);
    click(cx);
    assert!(log.borrow().is_empty());
}

#[gpui_kit::test]
fn space_and_enter_toggle_from_the_keyboard_and_the_ring_shows(cx: &mut TestAppContext) {
    let (owner, cx, log) = open(false, false, false, true, cx);
    click(cx);
    frames(&owner, cx, 2);
    assert_eq!(*log.borrow(), [true]);
    assert!(
        cx.debug_bounds("checkbox-ring").is_none(),
        "a click gives no ring: it is for the keyboard"
    );
    cx.simulate_keystrokes("space");
    assert_eq!(*log.borrow(), [true, false]);
    frames(&owner, cx, 3);
    assert!(
        cx.debug_bounds("checkbox-ring").is_some(),
        "the ring shows once a key is used"
    );
    cx.simulate_keystrokes("enter");
    assert_eq!(*log.borrow(), [true, false, true]);
}

#[gpui_kit::test]
fn under_reduce_motion_the_mark_is_there_whole_at_once_and_gone_at_once(cx: &mut TestAppContext) {
    let (owner, cx, _) = open(false, false, false, true, cx);
    assert_eq!(mark_width(cx), None);
    click(cx);
    frames(&owner, cx, 2);
    assert_eq!(mark_width(cx), Some(12.));
    click(cx);
    frames(&owner, cx, 2);
    assert_eq!(
        mark_width(cx),
        None,
        "no leaving mark: it jumps to its end state"
    );
}

#[gpui_kit::test]
fn with_motion_the_mark_grows_in_and_shrinks_out(cx: &mut TestAppContext) {
    let (owner, cx, _) = open(false, false, false, false, cx);
    click(cx);
    crate::motion::clock::advance(std::time::Duration::from_millis(30));
    frames(&owner, cx, 1);
    let early = mark_width(cx).expect("the mark is coming in");
    assert!(
        (6.0..12.0).contains(&early),
        "it starts at half size and grows: {early}"
    );
    crate::motion::clock::advance(std::time::Duration::from_millis(500));
    frames(&owner, cx, 3);
    assert_eq!(mark_width(cx), Some(12.));
    click(cx);
    crate::motion::clock::advance(std::time::Duration::from_millis(30));
    frames(&owner, cx, 1);
    let leaving = mark_width(cx).expect("the old mark is still on its way out");
    assert!(leaving < 12.);
    crate::motion::clock::advance(std::time::Duration::from_millis(500));
    frames(&owner, cx, 3);
    assert_eq!(mark_width(cx), None);
}

#[gpui_kit::test]
fn a_dash_replaces_a_tick_and_a_press_sinks_the_face(cx: &mut TestAppContext) {
    let (owner, cx, _) = open(true, false, false, false, cx);
    let rest = f32::from(cx.debug_bounds("checkbox-face").unwrap().size.width);
    assert_eq!(rest, 20.);
    let at = cx.debug_bounds("checkbox-box").unwrap().center();
    cx.simulate_event(gpui_kit::MouseDownEvent {
        position: at,
        modifiers: Default::default(),
        button: gpui_kit::MouseButton::Left,
        click_count: 1,
        first_mouse: false,
    });
    crate::motion::clock::advance(std::time::Duration::from_millis(300));
    frames(&owner, cx, 3);
    let held = f32::from(cx.debug_bounds("checkbox-face").unwrap().size.width);
    assert!(held < 19., "held: {held}");
    cx.simulate_event(gpui_kit::MouseUpEvent {
        position: at,
        modifiers: Default::default(),
        button: gpui_kit::MouseButton::Left,
        click_count: 1,
    });
    crate::motion::clock::advance(std::time::Duration::from_millis(600));
    frames(&owner, cx, 3);
    assert!((f32::from(cx.debug_bounds("checkbox-face").unwrap().size.width) - 20.).abs() < 0.3);
}
