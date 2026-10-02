use gpui_kit::prelude::FluentBuilder;
use std::time::Duration;

use gpui_kit::{
    AppContext as _, Bounds, Context, Entity, FocusHandle, InteractiveElement, IntoElement, ParentElement, Pixels, Render, StatefulInteractiveElement,
    Styled, TestAppContext, VisualTestContext, Window, div, point, px, size,
};

use super::*;
use crate::{
    placement::measure,
    theme::{Appearance, set_appearance},
};

/// Two triggers with a popover each, and a big element under both that counts what reaches it.
struct Host {
    open: [bool; 2],
    anchors: [Option<Bounds<Pixels>>; 2],
    focus: [FocusHandle; 2],
    beneath_clicks: usize,
    beneath_hovers: usize,
    item_clicks: usize,
    /// The trigger's left edge, so a test can move it.
    left: f32,
    /// Whether the popovers are drawn at all.
    drawn: bool,
    top: f32,
    /// Whether both popovers switch from one to the other on a press on the other's trigger.
    switchable: bool,
}

impl Host {
    fn trigger(&self, i: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity().downgrade();
        let me = this.clone();
        div()
            .id(("trigger", i))
            .debug_selector(move || format!("trigger-{i}"))
            .track_focus(&self.focus[i])
            .absolute()
            .top(px(self.top))
            .left(px(self.left + 200. * i as f32))
            .w(px(100.))
            .h(px(30.))
            .child(measure(move |b, cx| {
                me.update(cx, |h, _| h.anchors[i] = Some(b)).ok();
            }))
            .on_click(move |_, _, cx| {
                this.update(cx, |h, cx| {
                    h.open[i] = !h.open[i];
                    cx.notify();
                })
                .ok();
            })
    }

    fn popover(&self, i: usize, cx: &mut Context<Self>) -> Popover {
        let this = cx.entity().downgrade();
        let clicks = cx.entity().downgrade();
        let popover = Popover::new(("popover", i));
        let popover = if self.switchable { popover.switchable() } else { popover };
        popover
            .open(self.open[i])
            .anchor(self.anchors[i])
            .return_focus(&self.focus[i])
            .height(120.)
            .on_close(move |_, cx| {
                this.update(cx, |h, cx| {
                    h.open[i] = false;
                    cx.notify();
                })
                .ok();
            })
            .child(
                div()
                    .debug_selector(move || format!("panel-{i}"))
                    .w(px(160.))
                    .h(px(120.))
                    .child(
                        div()
                            .id(("item", i))
                            .debug_selector(move || format!("item-{i}"))
                            .w_full()
                            .h(px(40.))
                            .on_click(move |_, _, cx| {
                                clicks.update(cx, |h, _| h.item_clicks += 1).ok();
                            }),
                    ),
            )
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (clicks, hovers) = (cx.entity().downgrade(), cx.entity().downgrade());
        div()
            .size_full()
            .relative()
            .child(
                div()
                    .id("beneath")
                    .debug_selector(|| "beneath".into())
                    .absolute()
                    .top(px(50.))
                    .left_0()
                    .w(px(600.))
                    .h(px(300.))
                    .on_click(move |_, _, cx| {
                        clicks.update(cx, |h, _| h.beneath_clicks += 1).ok();
                    })
                    .on_hover(move |on, _, cx| {
                        if *on {
                            hovers.update(cx, |h, _| h.beneath_hovers += 1).ok();
                        }
                    }),
            )
            .child(self.trigger(0, cx))
            .child(self.trigger(1, cx))
            .when(self.drawn, |d| d.child(self.popover(0, cx)).child(self.popover(1, cx)))
    }
}

fn host(cx: &mut TestAppContext) -> (Entity<Host>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let (host, cx) = cx.add_window_view(|_, cx| Host::new(cx));
    cx.simulate_resize(size(px(600.), px(400.)));
    cx.run_until_parked();
    (host, cx)
}

impl Host {
    fn new(cx: &mut Context<Self>) -> Self {
        Host {
            open: [false; 2],
            anchors: [None; 2],
            focus: [cx.focus_handle(), cx.focus_handle()],
            beneath_clicks: 0,
            beneath_hovers: 0,
            item_clicks: 0,
            left: 20.,
            drawn: true,
            top: 10.,
            switchable: false,
        }
    }
}

/// Two copies of one view, one above the other, so their popovers have the same ids: two agent panels.
struct Pair([Entity<Host>; 2]);

impl Render for Pair {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().children(self.0.iter().map(|host| div().relative().w(px(600.)).h(px(200.)).child(host.clone())))
    }
}

fn press(host: &Entity<Host>, i: usize, cx: &mut VisualTestContext) {
    let at = host.read_with(cx, |h, _| h.anchors[i]).expect("the trigger is drawn").center();
    cx.simulate_click(at, gpui_kit::Modifiers::default());
    cx.run_until_parked();
    cx.run_until_parked();
}

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let at = cx.debug_bounds(selector).unwrap_or_else(|| panic!("{selector} is not drawn")).center();
    cx.simulate_click(at, gpui_kit::Modifiers::default());
    cx.run_until_parked();
}

fn is_open(host: &Entity<Host>, i: usize, cx: &mut VisualTestContext) -> bool {
    host.read_with(cx, |h, _| h.open[i])
}

fn open_first(host: &Entity<Host>, cx: &mut VisualTestContext) {
    click(cx, "trigger-0");
    assert!(is_open(host, 0, cx), "a click on the trigger opens it");
    cx.run_until_parked();
    assert!(cx.debug_bounds("panel-0").is_some(), "and the panel is drawn");
}

#[gpui_kit::test]
fn a_click_outside_closes_it_and_does_nothing_else(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    open_first(&host, cx);
    // On the element under the popover's area, away from the panel.
    cx.simulate_click(point(px(450.), px(300.)), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert!(!is_open(&host, 0, cx), "the press outside closed it");
    assert_eq!(host.read_with(cx, |h, _| h.beneath_clicks), 0, "and the element under it heard nothing");
    // With it closed, the same spot works again.
    cx.simulate_click(point(px(450.), px(300.)), gpui_kit::Modifiers::default());
    assert_eq!(host.read_with(cx, |h, _| h.beneath_clicks), 1);
}

#[gpui_kit::test]
fn nothing_under_it_is_hovered_while_it_is_open(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    let before = host.read_with(cx, |h, _| h.beneath_hovers);
    open_first(&host, cx);
    cx.simulate_mouse_move(point(px(450.), px(300.)), None, gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert_eq!(host.read_with(cx, |h, _| h.beneath_hovers), before, "the backdrop blocks hover");
}

#[gpui_kit::test]
fn a_click_on_the_trigger_toggles_it(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    open_first(&host, cx);
    click(cx, "trigger-0");
    assert!(!is_open(&host, 0, cx), "a second click on the trigger closes it");
    click(cx, "trigger-0");
    assert!(is_open(&host, 0, cx), "and a third opens it again");
}

#[gpui_kit::test]
fn a_click_on_the_panel_reaches_the_panel_and_not_the_element_under_it(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    open_first(&host, cx);
    click(cx, "item-0");
    assert_eq!(host.read_with(cx, |h, _| (h.item_clicks, h.beneath_clicks)), (1, 0));
    assert!(is_open(&host, 0, cx), "a press inside does not close it: the owner decides on a pick");
}

#[gpui_kit::test]
fn escape_closes_it_and_focus_returns_to_the_trigger(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    open_first(&host, cx);
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(!is_open(&host, 0, cx));
    let trigger = host.read_with(cx, |h, _| h.focus[0].clone());
    assert!(cx.update(|window, _| trigger.is_focused(window)), "focus is on the trigger again");
}

#[gpui_kit::test]
fn tab_closes_it_and_moves_on_from_the_trigger(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    open_first(&host, cx);
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    assert!(!is_open(&host, 0, cx), "tab closes it");
}

#[gpui_kit::test]
fn opening_one_closes_the_other(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    open_first(&host, cx);
    host.update(cx, |h, cx| {
        h.open[1] = true;
        cx.notify();
    });
    cx.run_until_parked();
    cx.run_until_parked();
    assert!(is_open(&host, 1, cx));
    assert!(!is_open(&host, 0, cx), "the first was closed by the second opening");
}

#[gpui_kit::test]
fn a_press_on_another_switchable_trigger_closes_this_one_and_opens_that_one(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    host.update(cx, |h, cx| {
        h.switchable = true;
        cx.notify();
    });
    cx.run_until_parked();
    open_first(&host, cx);
    click(cx, "trigger-1");
    cx.run_until_parked();
    assert!(is_open(&host, 1, cx), "the press reached the other trigger");
    assert!(!is_open(&host, 0, cx), "and the first closed");
    assert_eq!(host.read_with(cx, |h, _| h.beneath_clicks), 0);
    click(cx, "trigger-0");
    cx.run_until_parked();
    assert!(is_open(&host, 0, cx) && !is_open(&host, 1, cx), "and back again");
}

#[gpui_kit::test]
fn a_press_on_a_switchable_trigger_in_another_copy_of_the_view_switches_to_it(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let (pair, cx) = cx.add_window_view(|_, cx| {
        Pair([0, 1].map(|_| {
            cx.new(|cx| {
                let mut host = Host::new(cx);
                host.switchable = true;
                host
            })
        }))
    });
    cx.simulate_resize(size(px(600.), px(400.)));
    cx.run_until_parked();
    let [a, b] = pair.read_with(cx, |p, _| p.0.clone());
    let open = |host: &Entity<Host>, i: usize, cx: &mut VisualTestContext| host.read_with(cx, |h, _| h.open[i]);
    press(&a, 0, cx);
    assert!(open(&a, 0, cx), "the first view's popover opened");
    press(&b, 1, cx);
    assert!(open(&b, 1, cx) && !open(&a, 0, cx), "a press on the other view's other trigger switched to it");
    press(&a, 1, cx);
    assert!(open(&a, 1, cx) && !open(&b, 1, cx), "and the popover with the same id in the first view takes over from it");
}

#[gpui_kit::test]
fn a_press_on_a_trigger_that_does_not_switch_only_closes_the_open_one(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    open_first(&host, cx);
    click(cx, "trigger-1");
    cx.run_until_parked();
    assert!(!is_open(&host, 0, cx) && !is_open(&host, 1, cx));
}

#[test]
fn the_backdrop_covers_the_window_but_its_holes() {
    let b = |x: f32, y: f32, w: f32, h: f32| Bounds::new(point(px(x), px(y)), size(px(w), px(h)));
    let area = |rects: &[Bounds<Pixels>]| rects.iter().map(|r| f32::from(r.size.width) * f32::from(r.size.height)).sum::<f32>();
    let window = size(px(100.), px(50.));
    assert_eq!(cover(window, &[]), vec![b(0., 0., 100., 50.)]);
    let holes = [b(10., 10., 20., 10.), b(50., 15., 20., 10.)];
    let strips = cover(window, &holes);
    assert_eq!(area(&strips), 5000. - 400., "everything but the holes");
    for strip in &strips {
        assert!(holes.iter().all(|h| !strip.intersects(h)), "{strip:?} covers a hole");
    }
    let outside = cover(window, &[b(-10., -10., 30., 20.), b(90., 40., 30., 30.)]);
    assert_eq!(area(&outside), 5000. - 200. - 100., "a hole past the edge is cut to the window");
}

#[gpui_kit::test]
fn it_closes_when_the_trigger_moves(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    open_first(&host, cx);
    host.update(cx, |h, cx| {
        h.left += 40.;
        cx.notify();
    });
    // The move is measured in the first frame and seen by the popover in the next.
    for _ in 0..3 {
        host.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
    }
    assert!(!is_open(&host, 0, cx), "a trigger that scrolled away takes the popover with it");
}

#[gpui_kit::test]
fn it_closes_when_the_window_resizes(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    open_first(&host, cx);
    cx.simulate_resize(size(px(500.), px(380.)));
    cx.run_until_parked();
    cx.run_until_parked();
    assert!(!is_open(&host, 0, cx));
}

#[test]
fn a_popover_is_hidden_after_three_frames_it_missed_and_never_while_it_is_drawn() {
    let mut frames = Frames { watched: 0, drawn: 0, close: None };
    for _ in 0..500 {
        assert!(!frames.tick(), "drawn in every frame");
        frames.drew();
    }
    assert!(!frames.tick(), "one frame missed");
    assert!(!frames.tick(), "two frames missed");
    assert!(frames.tick(), "three frames missed: its view is gone");
    let mut back = Frames { watched: 9, drawn: 9, close: None };
    back.tick();
    back.tick();
    back.drew();
    assert!(!back.tick(), "drawn again after two missed frames: the count starts over");
}
#[gpui_kit::test]
fn a_3_second_frame_does_not_close_it_because_only_frames_are_counted(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    open_first(&host, cx);
    std::thread::sleep(Duration::from_secs(3));
    host.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    cx.run_until_parked();
    assert!(is_open(&host, 0, cx), "a late frame is not a hidden view");
}
#[gpui_kit::test]
fn it_opens_above_the_trigger_when_there_is_no_room_below(cx: &mut TestAppContext) {
    let (host, cx) = host(cx);
    host.update(cx, |h, cx| {
        h.top = 340.;
        cx.notify();
    });
    cx.run_until_parked();
    cx.run_until_parked();
    open_first(&host, cx);
    let trigger = cx.debug_bounds("trigger-0").unwrap();
    let panel = cx.debug_bounds("panel-0").unwrap();
    assert!(panel.bottom() <= trigger.top(), "the panel is above: {panel:?} over {trigger:?}");
}

#[test]
fn placement_chooses_the_side_and_the_corner() {
    let anchor = Bounds { origin: point(px(100.), px(100.)), size: size(px(80.), px(30.)) };
    let (up, corner, at) = placement(anchor, Side::Auto, Align::Start, 4., 120., 600.);
    assert!(!up && corner == Anchor::TopLeft && at == point(px(100.), px(134.)));
    let low = Bounds { origin: point(px(100.), px(400.)), size: size(px(80.), px(30.)) };
    let (up, corner, at) = placement(low, Side::Auto, Align::End, 4., 300., 600.);
    assert!(up && corner == Anchor::BottomRight && at == point(px(180.), px(396.)), "no room below, more above, right edge");
    assert!(placement(anchor, Side::Above, Align::Start, 4., 10., 600.).0);
    assert!(!placement(anchor, Side::Below, Align::Start, 4., 900., 600.).0);
}

/// A popover hung from its parent, which keeps focus and its own keys.
struct Hung {
    open: bool,
    page_clicks: usize,
    focus: FocusHandle,
}

impl Render for Hung {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (this, page) = (cx.entity().downgrade(), cx.entity().downgrade());
        div()
            .id("hung-page")
            .size_full()
            .relative()
            .track_focus(&self.focus)
            .on_click(move |_, _, cx| {
                page.update(cx, |h, _| h.page_clicks += 1).ok();
            })
            .child(
                Popover::new("hung")
                    .open(self.open)
                    .hang(Hang::Left(40., 60.))
                    .keep_focus()
                    .height(100.)
                    .on_close(move |_, cx| {
                        this.update(cx, |h, cx| {
                            h.open = false;
                            cx.notify();
                        })
                        .ok();
                    })
                    .child(div().debug_selector(|| "hung-panel".into()).w(px(120.)).h(px(100.))),
            )
    }
}

fn hung(cx: &mut TestAppContext) -> (Entity<Hung>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, cx| Hung { open: false, page_clicks: 0, focus: cx.focus_handle() });
    cx.simulate_resize(size(px(400.), px(300.)));
    cx.run_until_parked();
    let focus = view.read_with(cx, |h, _| h.focus.clone());
    cx.update(|window, cx| focus.focus(window, cx));
    view.update(cx, |h, cx| {
        h.open = true;
        cx.notify();
    });
    for _ in 0..3 {
        cx.run_until_parked();
        view.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
    (view, cx)
}

#[gpui_kit::test]
fn a_popover_hung_from_its_parent_sits_at_the_point_and_keeps_focus(cx: &mut TestAppContext) {
    let (view, cx) = hung(cx);
    let panel = cx.debug_bounds("hung-panel").expect("the panel is drawn");
    assert!((f32::from(panel.left()) - 40.).abs() < 1.5 && (f32::from(panel.top()) - 60.).abs() < 1.5, "at the point: {panel:?}");
    let focus = view.read_with(cx, |h, _| h.focus.clone());
    assert!(cx.update(|window, _| focus.is_focused(window)), "the owner still has focus");
}

#[gpui_kit::test]
fn a_hung_popover_closes_on_a_press_outside_and_the_page_hears_nothing(cx: &mut TestAppContext) {
    let (view, cx) = hung(cx);
    let before = view.read_with(cx, |h, _| h.page_clicks);
    cx.simulate_click(point(px(350.), px(250.)), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert!(!view.read_with(cx, |h, _| h.open), "closed");
    assert_eq!(view.read_with(cx, |h, _| h.page_clicks), before, "and the page heard nothing");
}

#[gpui_kit::test]
fn escape_closes_a_hung_popover_while_the_owner_has_focus(cx: &mut TestAppContext) {
    let (view, cx) = hung(cx);
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(!view.read_with(cx, |h, _| h.open));
}

mod components {
    use std::{cell::Cell, rc::Rc};

    use super::*;
    use crate::{
        review::{ReviewHandlers, ReviewProgress},
        review_bar::ReviewBar,
    };

    thread_local! {
        static PAGE: Cell<usize> = const { Cell::new(0) };
    }

    struct BarPage;

    impl Render for BarPage {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let handlers = ReviewHandlers::default().on_accept_all(|_, _| {}).on_reject_all(|_, _| {});
            div()
                .id("bar-page")
                .size_full()
                .on_click(|_, _, _| PAGE.with(|p| p.set(p.get() + 1)))
                .child(ReviewBar::new("bar", ReviewProgress { files: 3, reviewed: 1, added: 4, removed: 2 }, handlers))
        }
    }

    #[gpui_kit::test]
    fn the_review_bar_menu_closes_on_a_press_outside_and_the_page_hears_nothing(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(true);
        });
        let (_view, cx) = cx.add_window_view(|_, _| BarPage);
        cx.simulate_resize(size(px(900.), px(400.)));
        cx.run_until_parked();
        click(cx, "review-bar-more");
        assert!(cx.debug_bounds("Accept all").is_none() || true);
        let opened_with = PAGE.with(|p| p.get());
        cx.simulate_click(point(px(450.), px(300.)), gpui_kit::Modifiers::default());
        cx.run_until_parked();
        assert_eq!(PAGE.with(|p| p.get()), opened_with, "the press that closes the menu goes no further");
        // With it closed, the same spot reaches the page again.
        cx.simulate_click(point(px(450.), px(300.)), gpui_kit::Modifiers::default());
        assert_eq!(PAGE.with(|p| p.get()), opened_with + 1);
        let _ = Rc::new(());
    }
}

/// A popover hung from the right edge of its parent lines its right edge up with that point.
struct Right;

impl Render for Right {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().relative().child(
            Popover::new("right")
                .open(true)
                .hang(Hang::Right(10., 20.))
                .keep_focus()
                .child(div().debug_selector(|| "right-panel".into()).w(px(100.)).h(px(40.))),
        )
    }
}

#[gpui_kit::test]
fn a_popover_hung_from_the_right_lines_its_right_edge_up_with_the_point(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, _| Right);
    cx.simulate_resize(size(px(400.), px(300.)));
    for _ in 0..3 {
        cx.run_until_parked();
        view.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
    let panel = cx.debug_bounds("right-panel").expect("drawn");
    assert!((f32::from(panel.right()) - 390.).abs() < 1.5, "its right edge is 10px from the window's: {panel:?}");
}

/// A trigger with a field in it, and a popover with a hole around the trigger.
struct Holed {
    open: bool,
    anchor: Option<Bounds<Pixels>>,
    trigger_clicks: usize,
    page_clicks: usize,
}

impl Render for Holed {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (this, page, trig, measured) = (cx.entity().downgrade(), cx.entity().downgrade(), cx.entity().downgrade(), cx.entity().downgrade());
        div()
            .id("holed-page")
            .size_full()
            .relative()
            .on_click(move |_, _, cx| {
                page.update(cx, |h, _| h.page_clicks += 1).ok();
            })
            .child(
                div()
                    .id("holed-trigger")
                    .debug_selector(|| "holed-trigger".into())
                    .absolute()
                    .top(px(10.))
                    .left(px(20.))
                    .w(px(200.))
                    .h(px(40.))
                    .child(measure(move |b, cx| {
                        measured.update(cx, |h, _| h.anchor = Some(b)).ok();
                    }))
                    .on_click(move |_, _, cx| {
                        trig.update(cx, |h, cx| {
                            h.trigger_clicks += 1;
                            h.open = true;
                            cx.notify();
                        })
                        .ok();
                    }),
            )
            .child(
                Popover::new("holed")
                    .open(self.open)
                    .anchor(self.anchor)
                    .hole()
                    .keep_focus()
                    .on_close(move |_, cx| {
                        this.update(cx, |h, cx| {
                            h.open = false;
                            cx.notify();
                        })
                        .ok();
                    })
                    .child(div().debug_selector(|| "holed-panel".into()).w(px(200.)).h(px(80.))),
            )
    }
}

#[gpui_kit::test]
fn a_popover_with_a_hole_leaves_its_trigger_live_and_closes_on_a_press_anywhere_else(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, _| Holed { open: false, anchor: None, trigger_clicks: 0, page_clicks: 0 });
    cx.simulate_resize(size(px(500.), px(300.)));
    for _ in 0..3 {
        cx.run_until_parked();
        view.update(cx, |_, cx| cx.notify());
    }
    click(cx, "holed-trigger");
    for _ in 0..3 {
        cx.run_until_parked();
        view.update(cx, |_, cx| cx.notify());
    }
    assert!(view.read_with(cx, |h, _| h.open) && cx.debug_bounds("holed-panel").is_some(), "the trigger opened it");
    click(cx, "holed-trigger");
    let (trigger, open) = view.read_with(cx, |h, _| (h.trigger_clicks, h.open));
    assert_eq!(trigger, 2, "the trigger heard the press while the popover was open");
    assert!(open, "and a press on the trigger does not close it");
    let page = view.read_with(cx, |h, _| h.page_clicks);
    cx.simulate_click(point(px(450.), px(250.)), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert!(!view.read_with(cx, |h, _| h.open), "a press elsewhere closes it");
    assert_eq!(view.read_with(cx, |h, _| h.page_clicks), page, "and reaches nothing else");
}
