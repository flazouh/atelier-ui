use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Entity, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, size};

use super::*;
use gpui_kit::AppContext as _;
use crate::theme::{Appearance, set_appearance};

#[test]
fn the_stack_is_384_wide_and_16_short_of_a_narrow_window() {
    assert_eq!(stack_width(1200.), 384.);
    assert_eq!(stack_width(400.), 368.);
    assert_eq!(stack_width(20.), 0.);
}

#[test]
fn a_drag_lets_go_past_72px_or_520_per_second_and_follows_at_0_18() {
    assert!(!lets_go(72., 0.) && lets_go(73., 0.) && lets_go(-73., 0.));
    assert!(!lets_go(10., 520.) && lets_go(10., 521.) && lets_go(0., -600.));
    assert!((elastic(100.) - 18.).abs() < 1e-4 && (elastic(-50.) + 9.).abs() < 1e-4);
}

#[test]
fn the_newest_toasts_are_drawn_with_the_oldest_at_the_bottom_of_a_bottom_stack() {
    let all = [1, 2, 3, 4, 5];
    assert_eq!(drawn(&all, 4, ToastPosition::TopRight), [2, 3, 4, 5], "a top stack: the oldest first");
    assert_eq!(drawn(&all, 4, ToastPosition::BottomRight), [5, 4, 3, 2], "a bottom stack: the oldest last");
    assert_eq!(drawn(&all, 9, ToastPosition::TopLeft), all);
    assert!(drawn::<i32>(&[], 4, ToastPosition::TopLeft).is_empty());
}

fn open(position: ToastPosition, reduce: bool, cx: &mut TestAppContext) -> (Entity<ToastStack>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::motion::clock::freeze();
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
    });
    let (stack, cx) = cx.add_window_view(move |_, _| ToastStack::new("toasts").position(position).default_duration(Duration::from_secs(1)));
    cx.simulate_resize(size(px(800.), px(600.)));
    frames(&stack, cx, 3);
    (stack, cx)
}

fn frames(stack: &Entity<ToastStack>, cx: &mut VisualTestContext, n: usize) {
    for _ in 0..n {
        cx.run_until_parked();
        stack.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
}

fn show(stack: &Entity<ToastStack>, cx: &mut VisualTestContext, toast: Toast) -> SharedString {
    let id = stack.update(cx, |s, cx| s.show(toast, cx));
    frames(stack, cx, 3);
    id
}

fn ids(stack: &Entity<ToastStack>, cx: &mut VisualTestContext) -> Vec<String> {
    stack.read_with(cx, |s, _| s.toasts().iter().map(|t| t.id.to_string()).collect())
}

fn bounds(cx: &mut VisualTestContext, id: &str) -> gpui_kit::Bounds<Pixels> {
    cx.debug_bounds(Box::leak(format!("toast-{id}").into_boxed_str())).unwrap_or_else(|| panic!("toast {id} is not drawn"))
}

fn click(cx: &mut VisualTestContext, name: String) {
    let at = cx.debug_bounds(Box::leak(name.into_boxed_str())).expect("drawn").center();
    cx.simulate_click(at, Modifiers::default());
    cx.run_until_parked();
}

#[gpui_kit::test]
fn a_bottom_right_stack_sits_16px_from_the_right_and_24_from_the_bottom_with_the_newest_on_top(cx: &mut TestAppContext) {
    let (stack, cx) = open(ToastPosition::BottomRight, true, cx);
    show(&stack, cx, Toast::new("First").id("a").sticky());
    show(&stack, cx, Toast::new("Second").id("b").sticky());
    let (a, b) = (bounds(cx, "a"), bounds(cx, "b"));
    assert_eq!(f32::from(a.right()), 800. - EDGE_X);
    assert_eq!(f32::from(a.bottom()), 600. - EDGE_BOTTOM);
    assert_eq!(f32::from(a.size.width), 384.);
    assert!(b.bottom() <= a.top(), "the newest is above: {a:?} {b:?}");
    assert_eq!(f32::from(a.top() - b.bottom()), GAP, "8px apart");
}

#[gpui_kit::test]
fn a_top_left_stack_lists_the_oldest_first(cx: &mut TestAppContext) {
    let (stack, cx) = open(ToastPosition::TopLeft, true, cx);
    show(&stack, cx, Toast::new("First").id("a").sticky());
    show(&stack, cx, Toast::new("Second").id("b").sticky());
    let (a, b) = (bounds(cx, "a"), bounds(cx, "b"));
    assert_eq!((f32::from(a.left()), f32::from(a.top())), (EDGE_X, EDGE_TOP));
    assert!(b.top() >= a.bottom());
}

#[gpui_kit::test]
fn a_toast_goes_after_its_duration_and_a_sticky_one_stays(cx: &mut TestAppContext) {
    let (stack, cx) = open(ToastPosition::BottomRight, true, cx);
    show(&stack, cx, Toast::new("Default").id("d"));
    show(&stack, cx, Toast::new("Short").id("s").duration(Duration::from_millis(300)));
    show(&stack, cx, Toast::new("Sticky").id("k").sticky());
    cx.executor().advance_clock(Duration::from_millis(400));
    frames(&stack, cx, 2);
    assert_eq!(ids(&stack, cx), ["d", "k"], "the short one went");
    cx.executor().advance_clock(Duration::from_millis(800));
    frames(&stack, cx, 2);
    assert_eq!(ids(&stack, cx), ["k"], "the default is 1 s in this stack");
    cx.executor().advance_clock(Duration::from_secs(60));
    frames(&stack, cx, 2);
    assert_eq!(ids(&stack, cx), ["k"], "a sticky toast stays");
}

#[gpui_kit::test]
fn an_update_changes_a_toast_in_place_and_a_new_duration_restarts_its_time(cx: &mut TestAppContext) {
    let (stack, cx) = open(ToastPosition::BottomRight, true, cx);
    let id = show(&stack, cx, Toast::new("Publishing").id("p").status(ToastStatus::Loading).sticky());
    stack.update(cx, |s, cx| {
        s.update(&id, ToastPatch { title: Some("Done".into()), status: Some(ToastStatus::Success), duration: Some(Duration::from_millis(500)), ..Default::default() }, cx)
    });
    frames(&stack, cx, 2);
    let toast = stack.read_with(cx, |s, _| s.get("p").cloned()).expect("still there");
    assert_eq!((toast.title.as_ref(), toast.status), ("Done", ToastStatus::Success));
    cx.executor().advance_clock(Duration::from_millis(600));
    frames(&stack, cx, 2);
    assert!(ids(&stack, cx).is_empty(), "the sticky toast now has a time");
    stack.update(cx, |s, cx| s.update("nobody", ToastPatch::default(), cx));
}

#[gpui_kit::test]
fn the_close_button_takes_a_toast_away_and_a_stranger_id_does_nothing(cx: &mut TestAppContext) {
    let (stack, cx) = open(ToastPosition::BottomRight, true, cx);
    show(&stack, cx, Toast::new("One").id("a").sticky());
    show(&stack, cx, Toast::new("Two").id("b").sticky().dismissible(false));
    assert!(cx.debug_bounds("toast-close-b").is_none(), "no close button on a toast that cannot be dismissed");
    click(cx, "toast-close-a".into());
    frames(&stack, cx, 2);
    assert_eq!(ids(&stack, cx), ["b"]);
    stack.update(cx, |s, cx| s.dismiss("nobody", cx));
    assert_eq!(ids(&stack, cx), ["b"]);
}

#[gpui_kit::test]
fn the_action_button_says_which_toast_it_belongs_to(cx: &mut TestAppContext) {
    let (stack, cx) = open(ToastPosition::BottomRight, true, cx);
    let heard = Rc::new(RefCell::new(Vec::new()));
    let log = heard.clone();
    let sub = cx.update(|_, cx| cx.subscribe(&stack, move |_, event: &ToastEvent, _| match event {
        ToastEvent::Action(id) => log.borrow_mut().push(id.to_string()),
    }));
    show(&stack, cx, Toast::new("Undo?").id("u").description("Deleted a file").action("Undo").sticky());
    click(cx, "toast-action-u".into());
    assert_eq!(*heard.borrow(), ["u"]);
    assert_eq!(ids(&stack, cx), ["u"], "the action alone does not close it");
    drop(sub);
}

#[gpui_kit::test]
fn only_the_newest_max_visible_toasts_are_drawn(cx: &mut TestAppContext) {
    let (stack, cx) = open(ToastPosition::BottomRight, true, cx);
    stack.update(cx, |s, _| s.max_visible = 2);
    for id in ["a", "b", "c"] {
        show(&stack, cx, Toast::new(id).id(id).sticky());
    }
    assert!(cx.debug_bounds("toast-a").is_none(), "the oldest is kept but not drawn");
    assert!(cx.debug_bounds("toast-b").is_some() && cx.debug_bounds("toast-c").is_some());
    assert_eq!(ids(&stack, cx).len(), 3);
}

#[gpui_kit::test]
fn a_limit_pushes_the_oldest_out(cx: &mut TestAppContext) {
    let (stack, cx) = open(ToastPosition::BottomRight, true, cx);
    stack.update(cx, |s, _| s.limit = Some(2));
    for id in ["a", "b", "c"] {
        show(&stack, cx, Toast::new(id).id(id).sticky());
    }
    assert_eq!(ids(&stack, cx), ["b", "c"]);
}

#[gpui_kit::test]
fn a_toast_dragged_past_72px_leaves_and_one_dragged_less_springs_back(cx: &mut TestAppContext) {
    let (stack, cx) = open(ToastPosition::BottomRight, false, cx);
    show(&stack, cx, Toast::new("Near").id("n").sticky());
    show(&stack, cx, Toast::new("Far").id("f").sticky());
    crate::motion::clock::advance(Duration::from_millis(600));
    frames(&stack, cx, 3);
    let near = bounds(cx, "n").center();
    cx.simulate_mouse_down(near, MouseButton::Left, Modifiers::default());
    // 40px in 120 ms is 333px/s: slower than a throw.
    crate::motion::clock::advance(Duration::from_millis(120));
    cx.simulate_mouse_move(point(near.x - px(40.), near.y), MouseButton::Left, Modifiers::default());
    frames(&stack, cx, 2);
    let moved = bounds(cx, "n").center().x;
    assert!(f32::from(near.x - moved) > 3. && f32::from(near.x - moved) < 12., "it follows at 0.18 of 40: {}", f32::from(near.x - moved));
    cx.simulate_mouse_up(point(near.x - px(40.), near.y), MouseButton::Left, Modifiers::default());
    frames(&stack, cx, 2);
    assert_eq!(ids(&stack, cx).len(), 2, "40px is not enough");
    crate::motion::clock::advance(Duration::from_millis(700));
    frames(&stack, cx, 3);
    assert!(f32::from(bounds(cx, "n").center().x - near.x).abs() < 1., "and it is back in place");

    let far = bounds(cx, "f").center();
    cx.simulate_mouse_down(far, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(point(far.x + px(100.), far.y), MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(point(far.x + px(100.), far.y), MouseButton::Left, Modifiers::default());
    frames(&stack, cx, 2);
    assert_eq!(ids(&stack, cx), ["n"], "100px sends it away");
}

#[gpui_kit::test]
fn a_toast_that_goes_leaves_a_ghost_that_runs_out_and_reduce_motion_leaves_none(cx: &mut TestAppContext) {
    let (stack, cx) = open(ToastPosition::BottomRight, false, cx);
    show(&stack, cx, Toast::new("Bye").id("a").sticky());
    frames(&stack, cx, 2);
    click(cx, "toast-close-a".into());
    frames(&stack, cx, 1);
    assert_eq!(stack.read_with(cx, |s, _| s.leaving.len()), 1, "a ghost slides away");
    crate::motion::clock::advance(Duration::from_millis(300));
    frames(&stack, cx, 2);
    assert!(stack.read_with(cx, |s, _| s.leaving.is_empty()), "and it ends");

    let (stack, cx) = open(ToastPosition::BottomRight, true, cx);
    show(&stack, cx, Toast::new("Bye").id("a").sticky());
    assert_eq!(stack.read_with(cx, |s, _| s.items[0].enter.value()), 1., "it is in place at once");
    click(cx, "toast-close-a".into());
    assert!(stack.read_with(cx, |s, _| s.leaving.is_empty()), "no ghost");
}

#[gpui_kit::test]
fn a_change_of_status_swaps_the_words_and_the_swap_runs_out(cx: &mut TestAppContext) {
    let (stack, cx) = open(ToastPosition::BottomRight, false, cx);
    show(&stack, cx, Toast::new("Publishing").id("p").status(ToastStatus::Loading).sticky());
    stack.update(cx, |s, cx| s.update("p", ToastPatch { title: Some("Published".into()), status: Some(ToastStatus::Success), ..Default::default() }, cx));
    frames(&stack, cx, 1);
    assert!(stack.read_with(cx, |s, _| s.items[0].swap.is_some()), "the old words are still fading out");
    crate::motion::clock::advance(Duration::from_millis(400));
    frames(&stack, cx, 3);
    assert!(stack.read_with(cx, |s, _| s.items[0].swap.is_none()));
    // The same words again is no change.
    stack.update(cx, |s, cx| s.update("p", ToastPatch { title: Some("Published".into()), ..Default::default() }, cx));
    assert!(stack.read_with(cx, |s, _| s.items[0].swap.is_none()));
}

struct Boxed {
    stack: Entity<ToastStack>,
}

impl gpui_kit::Render for Boxed {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        // A small box away from the window's corner: the stack still goes to the window's.
        div().size_full().child(div().relative().ml(px(120.)).mt(px(80.)).w(px(300.)).h(px(200.)).child(self.stack.clone()))
    }
}

#[gpui_kit::test]
fn the_stack_is_fixed_to_the_window_whatever_box_it_is_put_in(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
        crate::motion::clock::freeze();
    });
    let (boxed, cx) = cx.add_window_view(|_, cx| Boxed { stack: cx.new(|_| ToastStack::new("boxed")) });
    cx.simulate_resize(size(px(800.), px(600.)));
    let stack = boxed.read_with(cx, |b, _| b.stack.clone());
    show(&stack, cx, Toast::new("Fixed").id("a").sticky());
    let at = bounds(cx, "a");
    assert_eq!((f32::from(at.right()), f32::from(at.bottom())), (800. - EDGE_X, 600. - EDGE_BOTTOM));
}

#[test]
fn the_status_discs_show_against_the_card_and_their_glyphs_read_on_them_in_every_theme() {
    use crate::theme::{MARK_CONTRAST, contrast};
    for theme in crate::themes::all() {
        for status in [ToastStatus::Success, ToastStatus::Error] {
            let (glyph, disc) = status.tones(theme, theme.card);
            assert!(contrast(disc, theme.card) >= 1.08, "{} {status:?}: the disc shows: {:.3}", theme.name, contrast(disc, theme.card));
            assert!(contrast(glyph, disc) >= MARK_CONTRAST, "{} {status:?}: the glyph reads: {:.2}", theme.name, contrast(glyph, disc));
        }
    }
    // Where the web's 10% is enough, it stays 10%.
    let bright = gpui_kit::Rgba { r: 0.06, g: 0.72, b: 0.51, a: 1. }.into();
    let white = gpui_kit::Rgba { r: 1., g: 1., b: 1., a: 1. }.into();
    let (_, disc) = disc_for(&crate::theme::Theme::light(), bright, 0.1, white);
    assert_eq!(disc, crate::theme::mix(white, bright, 0.1));
}
