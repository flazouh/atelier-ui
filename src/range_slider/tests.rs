use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    Context, Entity, IntoElement, ParentElement, Render, Styled, TestAppContext, VisualTestContext,
    Window, div, point, px, size,
};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn a_value_snaps_to_the_grid_and_to_the_maximum_when_the_step_does_not_divide_the_range() {
    assert_eq!(snap(43., 0., 100., 5.), 45.);
    assert_eq!(snap(42., 0., 100., 5.), 40.);
    assert_eq!(
        snap(-9., 0., 100., 5.),
        0.,
        "below the range is the minimum"
    );
    assert_eq!(snap(300., 0., 100., 5.), 100.);
    // 0 to 10 by 4: 0, 4, 8, and 10 is legal too, so a pointer near the end does not fall back to 8.
    assert_eq!(snap(9.5, 0., 10., 4.), 10.);
    assert_eq!(snap(8.2, 0., 10., 4.), 8.);
    assert_eq!(snap(6.1, 0., 10., 4.), 8.);
    assert_eq!(
        snap(5., 3., 3., 1.),
        3.,
        "an empty range has one legal point"
    );
    assert_eq!(
        snap(2.7, 0., 10., 0.),
        2.7,
        "a step of zero is only a clamp"
    );
    assert_eq!(snap(0.3 + 1e-7, 0., 1., 0.1), 0.3, "no floating drift");
}

#[test]
fn the_value_is_a_percent_of_the_range_and_an_empty_range_is_zero() {
    assert_eq!(percent(50., 0., 200.), 25.);
    assert_eq!(percent(-5., 0., 10.), 0.);
    assert_eq!(percent(1., 1., 1.), 0.);
}

#[test]
fn there_is_a_dot_at_each_step_unless_there_are_too_many() {
    assert_eq!(ticks(0., 100., 25.), vec![0., 25., 50., 75., 100.]);
    assert_eq!(
        ticks(0., 10., 4.),
        vec![0., 4., 8.],
        "the dots stop at the last whole step"
    );
    assert_eq!(
        ticks(0., 0.3, 0.1).len(),
        4,
        "0.3 / 0.1 is 2.9999999999999996 and still counts three steps"
    );
    assert!(ticks(0., 100., 1.).is_empty(), "101 dots are too many");
    assert!(ticks(0., 100., 0.).is_empty() && ticks(5., 5., 1.).is_empty());
}

#[test]
fn the_keys_ask_for_a_step_ten_steps_or_an_end() {
    assert_eq!(key_value("right", 40., 0., 100., 5.), Some(45.));
    assert_eq!(key_value("down", 40., 0., 100., 5.), Some(35.));
    assert_eq!(key_value("pageup", 40., 0., 100., 5.), Some(90.));
    assert_eq!(key_value("pagedown", 40., 0., 100., 5.), Some(-10.));
    assert_eq!(key_value("home", 40., 0., 100., 5.), Some(0.));
    assert_eq!(key_value("end", 40., 0., 100., 5.), Some(100.));
    assert_eq!(key_value("a", 40., 0., 100., 5.), None);
}

#[test]
fn the_handle_starts_eight_pixels_in_and_the_fill_ends_at_the_handle() {
    let (handle, fill) = geometry(292., 0.);
    assert_eq!(handle, 8.);
    assert!(
        (fill - (-288. + 14.)).abs() < 1e-3,
        "the fill sits a whole track to the left, 14px in: {fill}"
    );
    let (handle, fill) = geometry(292., 50.);
    assert_eq!(handle, 8. + 272. * 0.5);
    assert!((fill - (-144. + 14. - 8.)).abs() < 1e-3);
    let (handle, fill) = geometry(292., 100.);
    assert_eq!(
        (handle, fill),
        (280., 0.),
        "at the end the fill is where it stands"
    );
    assert_eq!(
        geometry(10., 50.).0,
        8.,
        "a track narrower than the inset does not send the handle backwards"
    );
}

struct Owner {
    value: f32,
    disabled: bool,
    compact: bool,
    log: Rc<RefCell<Vec<f32>>>,
    ends: Rc<RefCell<Vec<f32>>>,
}

impl Render for Owner {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (this, log, ends) = (cx.entity().downgrade(), self.log.clone(), self.ends.clone());
        div().w(px(300.)).p(px(20.)).child(
            RangeSlider::new("rs", self.value)
                .step(5.)
                .disabled(self.disabled)
                .compact(self.compact)
                .on_end(move |v, _, _| ends.borrow_mut().push(v))
                .on_change(move |v, _, cx| {
                    log.borrow_mut().push(v);
                    this.update(cx, |o, cx| {
                        o.value = v;
                        cx.notify();
                    })
                    .ok();
                }),
        )
    }
}

fn open(
    value: f32,
    disabled: bool,
    reduce: bool,
    cx: &mut TestAppContext,
) -> (Entity<Owner>, &mut VisualTestContext, Rc<RefCell<Vec<f32>>>) {
    let (owner, cx, log, _) = open_as(value, disabled, reduce, false, cx);
    (owner, cx, log)
}

type Log = Rc<RefCell<Vec<f32>>>;

fn open_as(
    value: f32,
    disabled: bool,
    reduce: bool,
    compact: bool,
    cx: &mut TestAppContext,
) -> (Entity<Owner>, &mut VisualTestContext, Log, Log) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::motion::clock::freeze();
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
    });
    let (log, ends) = (
        Rc::new(RefCell::new(Vec::new())),
        Rc::new(RefCell::new(Vec::new())),
    );
    let (seen, ended) = (log.clone(), ends.clone());
    let (owner, cx) = cx.add_window_view(move |_, _| Owner {
        value,
        disabled,
        compact,
        log: seen,
        ends: ended,
    });
    cx.simulate_resize(size(px(400.), px(200.)));
    frames(&owner, cx, 3);
    (owner, cx, log, ends)
}

fn frames(owner: &Entity<Owner>, cx: &mut VisualTestContext, n: usize) {
    for _ in 0..n {
        cx.run_until_parked();
        owner.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
}

/// The window x of a value on the track (the owner is 300 wide with 20px padding: the track is 260 wide from x = 20).
fn x_of(value: f32) -> f32 {
    20. + 260. * value / 100.
}

fn handle_x(cx: &mut VisualTestContext) -> f32 {
    f32::from(cx.debug_bounds("range-handle").unwrap().left())
}

#[gpui_kit::test]
fn a_press_on_the_track_moves_the_handle_there_snapped_to_the_step(cx: &mut TestAppContext) {
    let (owner, cx, log) = open(40., false, true, cx);
    cx.simulate_click(
        point(px(x_of(73.)), px(40.)),
        gpui_kit::Modifiers::default(),
    );
    frames(&owner, cx, 2);
    assert_eq!(log.borrow().first().copied(), Some(75.));
    let (expected, _) = geometry(260., 75.);
    assert!(
        (handle_x(cx) - (20. + expected)).abs() < 0.6,
        "under Reduce Motion the handle is there at once"
    );
}

#[gpui_kit::test]
fn a_drag_goes_on_outside_the_track_and_stops_at_the_ends(cx: &mut TestAppContext) {
    let (owner, cx, log) = open(40., false, true, cx);
    cx.simulate_mouse_down(
        point(px(x_of(40.)), px(40.)),
        gpui_kit::MouseButton::Left,
        Default::default(),
    );
    cx.simulate_mouse_move(
        point(px(x_of(60.)), px(40.)),
        gpui_kit::MouseButton::Left,
        Default::default(),
    );
    cx.simulate_mouse_move(
        point(px(390.), px(160.)),
        gpui_kit::MouseButton::Left,
        Default::default(),
    );
    frames(&owner, cx, 2);
    assert_eq!(
        log.borrow().last().copied(),
        Some(100.),
        "the pointer left the track, and the value is the end"
    );
    cx.simulate_mouse_up(
        point(px(390.), px(160.)),
        gpui_kit::MouseButton::Left,
        Default::default(),
    );
    frames(&owner, cx, 2);
    let count = log.borrow().len();
    cx.simulate_mouse_move(
        point(px(x_of(10.)), px(40.)),
        gpui_kit::MouseButton::Left,
        Default::default(),
    );
    frames(&owner, cx, 2);
    assert_eq!(
        log.borrow().len(),
        count,
        "after the release the pointer moves nothing"
    );
}

#[gpui_kit::test]
fn the_keys_move_the_handle_from_where_it_is(cx: &mut TestAppContext) {
    let (owner, cx, log) = open(40., false, true, cx);
    cx.simulate_click(
        point(px(x_of(40.)), px(40.)),
        gpui_kit::Modifiers::default(),
    );
    frames(&owner, cx, 2);
    let before = log.borrow().len();
    cx.simulate_keystrokes("right");
    cx.simulate_keystrokes("pagedown");
    cx.simulate_keystrokes("end");
    cx.simulate_keystrokes("home");
    let keys: Vec<f32> = log.borrow()[before..].to_vec();
    assert_eq!(
        keys,
        [45., 0., 100., 0.],
        "45 after a step, 0 after ten steps down from it, then the ends"
    );
}

#[gpui_kit::test]
fn a_disabled_slider_does_not_answer(cx: &mut TestAppContext) {
    let (owner, cx, log) = open(40., true, true, cx);
    cx.simulate_click(
        point(px(x_of(80.)), px(40.)),
        gpui_kit::Modifiers::default(),
    );
    cx.simulate_keystrokes("right");
    frames(&owner, cx, 2);
    assert!(log.borrow().is_empty());
}

#[gpui_kit::test]
fn with_motion_the_handle_glides_and_a_grab_stretches_it(cx: &mut TestAppContext) {
    let (owner, cx, _) = open(0., false, false, cx);
    let rest = f32::from(cx.debug_bounds("range-handle").unwrap().size.height);
    assert_eq!(rest, 24.);
    cx.simulate_mouse_down(
        point(px(x_of(80.)), px(40.)),
        gpui_kit::MouseButton::Left,
        Default::default(),
    );
    frames(&owner, cx, 1);
    let early = handle_x(cx);
    let (end, _) = geometry(260., 80.);
    assert!(
        early < 20. + end - 10.,
        "a frame after the press the handle has not arrived: {early}"
    );
    crate::motion::clock::advance(std::time::Duration::from_millis(600));
    frames(&owner, cx, 3);
    assert!(
        (handle_x(cx) - (20. + end)).abs() < 0.6,
        "it settles on the value"
    );
    let grabbed = f32::from(cx.debug_bounds("range-handle").unwrap().size.height);
    assert!(
        grabbed > 30.,
        "held, the handle stands 135% tall: {grabbed}"
    );
    cx.simulate_mouse_up(
        point(px(x_of(80.)), px(40.)),
        gpui_kit::MouseButton::Left,
        Default::default(),
    );
    crate::motion::clock::advance(std::time::Duration::from_millis(900));
    frames(&owner, cx, 3);
    assert!(
        (f32::from(cx.debug_bounds("range-handle").unwrap().size.height) - 24.).abs() < 0.3,
        "and back to 24 once let go"
    );
}

#[test]
fn a_compact_knob_stays_on_the_rail_and_the_fill_ends_at_its_centre() {
    use super::helpers::compact_geometry;
    use super::types::KNOB;
    let width = 160.;
    for percent in [0., 23., 50., 100.] {
        let (knob_left, fill) = compact_geometry(width, percent);
        assert!(
            knob_left >= 0. && knob_left + KNOB <= width,
            "{percent}%: the knob stays inside: {knob_left}"
        );
        assert!(
            (fill - (knob_left + KNOB / 2.)).abs() < 1e-4,
            "{percent}%: the fill ends at the knob's centre"
        );
    }
    assert_eq!(compact_geometry(width, 0.).0, 0.);
    assert_eq!(compact_geometry(width, 100.).0, width - KNOB);
}

#[gpui_kit::test]
fn zoomed_in_the_compact_fill_and_knob_stay_on_the_track(cx: &mut TestAppContext) {
    crate::scale::set_zoom(2.);
    let (owner, cx, _, _) = open_as(100., false, true, true, cx);
    frames(&owner, cx, 3);
    let track = cx.debug_bounds("range-track").unwrap();
    let (fill, knob) = (
        cx.debug_bounds("range-fill").unwrap(),
        cx.debug_bounds("range-handle").unwrap(),
    );
    crate::scale::set_zoom(1.);
    assert!(
        fill.right() <= track.right() + px(0.5),
        "the fill ends on the track: {fill:?} in {track:?}"
    );
    assert!(
        knob.right() <= track.right() + px(0.5),
        "the knob stays on the track: {knob:?} in {track:?}"
    );
    assert!(
        knob.right() > track.right() - px(2.),
        "at the maximum the knob is at the end: {knob:?} in {track:?}"
    );
}

#[gpui_kit::test]
fn zoomed_in_the_full_slider_handle_stays_on_the_track(cx: &mut TestAppContext) {
    crate::scale::set_zoom(2.);
    let (owner, cx, _, _) = open_as(100., false, true, false, cx);
    frames(&owner, cx, 3);
    let track = cx.debug_bounds("range-track").unwrap();
    let handle = cx.debug_bounds("range-handle").unwrap();
    crate::scale::set_zoom(1.);
    assert!(
        handle.right() < track.right(),
        "the handle stays on the track: {handle:?} in {track:?}"
    );
}

#[gpui_kit::test]
fn a_drag_ends_once_on_the_release_and_each_key_is_an_end(cx: &mut TestAppContext) {
    let (owner, cx, log, ends) = open_as(40., false, true, true, cx);
    cx.simulate_mouse_down(
        point(px(x_of(40.)), px(30.)),
        gpui_kit::MouseButton::Left,
        Default::default(),
    );
    cx.simulate_mouse_move(
        point(px(x_of(60.)), px(30.)),
        gpui_kit::MouseButton::Left,
        Default::default(),
    );
    cx.simulate_mouse_move(
        point(px(x_of(70.)), px(30.)),
        gpui_kit::MouseButton::Left,
        Default::default(),
    );
    frames(&owner, cx, 2);
    assert!(
        log.borrow().len() >= 2,
        "the drag reports as it goes: {:?}",
        log.borrow()
    );
    assert!(
        ends.borrow().is_empty(),
        "and has not ended while held: {:?}",
        ends.borrow()
    );
    cx.simulate_mouse_up(
        point(px(x_of(70.)), px(30.)),
        gpui_kit::MouseButton::Left,
        Default::default(),
    );
    frames(&owner, cx, 2);
    assert_eq!(
        ends.borrow().as_slice(),
        &[70.],
        "the release ends it at the value it reached"
    );
    cx.simulate_keystrokes("right");
    frames(&owner, cx, 2);
    assert_eq!(
        ends.borrow().as_slice(),
        &[70., 75.],
        "a key is a whole change"
    );
}
