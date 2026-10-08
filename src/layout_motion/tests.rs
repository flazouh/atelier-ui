use gpui_kit::{
    Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled,
    TestAppContext, VisualTestContext, Window, div, point, px, size,
};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn a_child_is_drawn_where_it_stood_until_the_spring_has_run() {
    // It stood at x = 100 and wore no offset; the layout now puts it at 60: it is drawn 40 to the right of that.
    assert_eq!(
        start_offset(
            point(px(100.), px(0.)),
            point(px(0.), px(0.)),
            point(px(60.), px(0.))
        ),
        point(px(40.), px(0.))
    );
    // It was still gliding (wearing 10) when the layout moved it again from 100 to 60: it stays where it was seen, at 110.
    assert_eq!(
        start_offset(
            point(px(100.), px(20.)),
            point(px(10.), px(-5.)),
            point(px(60.), px(20.))
        ),
        point(px(50.), px(-5.))
    );
}

#[test]
fn rounding_in_layout_is_not_a_move() {
    assert!(!moved(point(px(10.), px(10.)), point(px(10.3), px(9.8))));
    assert!(moved(point(px(10.), px(10.)), point(px(11.), px(10.))));
    assert!(moved(point(px(10.), px(10.)), point(px(10.), px(8.))));
}

/// A row of two boxes: the first is `first` wide, and the second follows it.
struct Row {
    first: f32,
}

impl Render for Row {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .p(px(10.))
            .child(
                div()
                    .debug_selector(|| "first".into())
                    .w(px(self.first))
                    .h(px(20.)),
            )
            .child(shifted(
                "second",
                div()
                    .id("second")
                    .debug_selector(|| "second".into())
                    .w(px(30.))
                    .h(px(20.)),
            ))
    }
}

fn open(reduce: bool, cx: &mut TestAppContext) -> (Entity<Row>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::motion::clock::freeze();
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
    });
    let (row, cx) = cx.add_window_view(|_, _| Row { first: 100. });
    cx.simulate_resize(size(px(400.), px(100.)));
    frames(&row, cx, 3);
    (row, cx)
}

fn frames(row: &Entity<Row>, cx: &mut VisualTestContext, n: usize) {
    for _ in 0..n {
        cx.run_until_parked();
        row.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
}

fn second_x(cx: &mut VisualTestContext) -> f32 {
    f32::from(cx.debug_bounds("second").unwrap().left())
}

#[gpui_kit::test]
fn with_motion_a_sibling_that_the_layout_moves_glides_to_its_new_place(cx: &mut TestAppContext) {
    let (row, cx) = open(false, cx);
    assert_eq!(second_x(cx), 110.);
    row.update(cx, |r, cx| {
        r.first = 40.;
        cx.notify();
    });
    cx.run_until_parked();
    let first_frame = second_x(cx);
    assert!(
        first_frame > 100. && first_frame <= 110.,
        "drawn where it stood in the frame the layout moved it: {first_frame}"
    );
    crate::motion::clock::advance(std::time::Duration::from_millis(80));
    frames(&row, cx, 1);
    let between = second_x(cx);
    assert!(
        between < first_frame && between > 50.,
        "on its way: {between}"
    );
    crate::motion::clock::advance(std::time::Duration::from_millis(1200));
    frames(&row, cx, 3);
    assert!(
        (second_x(cx) - 50.).abs() < 0.3,
        "it settles on the layout place, 10 + 40: {}",
        second_x(cx)
    );
}

#[gpui_kit::test]
fn under_reduce_motion_the_sibling_is_at_its_new_place_at_once(cx: &mut TestAppContext) {
    let (row, cx) = open(true, cx);
    row.update(cx, |r, cx| {
        r.first = 40.;
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(second_x(cx), 50.);
}

#[gpui_kit::test]
fn a_child_that_does_not_move_wears_no_offset_and_a_glide_carries_its_hit_box(
    cx: &mut TestAppContext,
) {
    let (row, cx) = open(false, cx);
    frames(&row, cx, 3);
    assert_eq!(second_x(cx), 110.);
    row.update(cx, |r, cx| {
        r.first = 40.;
        cx.notify();
    });
    cx.run_until_parked();
    // A click where the child is drawn now reaches it; the place it left does not.
    let drawn = cx.debug_bounds("second").unwrap();
    assert!(drawn.contains(&point(drawn.left() + px(5.), drawn.top() + px(5.))));
}
