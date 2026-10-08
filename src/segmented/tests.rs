use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    Entity, Hsla, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext,
    VisualTestContext, Window, div, px,
};

use super::*;
use crate::{
    motion,
    theme::{Appearance, contrast, set_appearance},
};

#[test]
fn a_segment_is_a_sm_button_with_no_track_round_it() {
    assert_eq!(
        (SEGMENT_HEIGHT, SEGMENT_PAD, TEXT, LINE, GAP),
        (28., 10., 11., 16., 2.)
    );
}
#[test]
fn a_chosen_segment_wears_the_secondary_fill_and_the_others_the_ghost_fill() {
    use crate::button::{ButtonVariant, colors};
    for appearance in [Appearance::Light, Appearance::Dark] {
        let theme = match appearance {
            Appearance::Light => Theme::light(),
            _ => Theme::dark(),
        };
        let near = |a: Hsla, b: Hsla| {
            (a.l - b.l).abs() < 1e-3
                && (a.s - b.s).abs() < 1e-3
                && (a.h - b.h).abs() < 1e-3
                && (a.a - b.a).abs() < 1e-3
        };
        assert!(near(
            segment_fill(&theme, 1., 0.),
            colors(ButtonVariant::Secondary, &theme, 0., false).0
        ));
        assert!(near(
            segment_fill(&theme, 0., 0.),
            colors(ButtonVariant::Ghost, &theme, 0., false).0
        ));
        assert!(
            near(
                segment_fill(&theme, 0., 1.),
                colors(ButtonVariant::Ghost, &theme, 1., false).0
            ),
            "hover lights a ghost segment"
        );
        assert!(near(segment_text(&theme, 0., 0.), theme.muted_foreground));
        assert!(near(segment_text(&theme, 1., 0.), theme.foreground));
        assert!(
            near(segment_text(&theme, 0., 1.), theme.foreground),
            "hover takes the text to the foreground"
        );
        assert!(
            contrast(
                segment_text(&theme, 1., 0.),
                segment_fill(&theme, 1., 0.).blend(theme.card)
            ) >= 4.5
        );
    }
}
#[test]
fn a_pressed_pill_shrinks_to_95_percent() {
    assert_eq!(pill_inset(0.), (0., 0.));
    let (across, down) = pill_inset(1.);
    assert!(
        (across - 0.025).abs() < 1e-6,
        "2.5% of the width on each side"
    );
    assert!(
        (down - 0.7).abs() < 1e-6,
        "0.7px off the top and bottom of 28"
    );
}

struct Host {
    selected: usize,
    log: Rc<RefCell<Vec<usize>>>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        let log = self.log.clone();
        div().p(px(20.)).child(
            Segmented::new(
                "seg",
                [
                    Segment::new("Centered"),
                    Segment::new("Row").cap("⌘L"),
                    Segment::new("Grid"),
                ],
                self.selected,
            )
            .debug_name("seg")
            .on_change(move |i, _, cx| {
                log.borrow_mut().push(i);
                this.update(cx, |h, cx| {
                    h.selected = i;
                    cx.notify();
                });
            }),
        )
    }
}

fn open(
    cx: &mut TestAppContext,
) -> (
    Entity<Host>,
    &mut VisualTestContext,
    Rc<RefCell<Vec<usize>>>,
) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let l = log.clone();
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let (host, cx) = cx.add_window_view(move |_, _| Host {
        selected: 0,
        log: l,
    });
    cx.run_until_parked();
    (host, cx, log)
}

fn centre(cx: &mut VisualTestContext, name: &'static str) -> gpui_kit::Point<gpui_kit::Pixels> {
    cx.debug_bounds(name)
        .unwrap_or_else(|| panic!("no {name}"))
        .center()
}

#[gpui_kit::test]
fn the_segments_are_28_tall_with_a_2px_gap_and_no_track(cx: &mut TestAppContext) {
    let (_, cx, _) = open(cx);
    let track = cx.debug_bounds("seg").unwrap();
    assert_eq!(
        f32::from(track.size.height),
        SEGMENT_HEIGHT,
        "no padding round the segments"
    );
    for name in ["seg-0", "seg-1", "seg-2"] {
        let b = cx.debug_bounds(name).unwrap();
        assert_eq!(f32::from(b.size.height), SEGMENT_HEIGHT);
    }
    let first = cx.debug_bounds("seg-0").unwrap();
    assert_eq!(
        first.origin.x, track.origin.x,
        "the first segment starts at the edge"
    );
    let second = cx.debug_bounds("seg-1").unwrap();
    assert_eq!(f32::from(second.left() - first.right()), GAP);
}

#[gpui_kit::test]
fn a_click_chooses_a_segment_once_and_the_chosen_one_does_not_fire(cx: &mut TestAppContext) {
    let (host, cx, log) = open(cx);
    let at = centre(cx, "seg-2");
    cx.simulate_click(at, Modifiers::default());
    assert_eq!(*log.borrow(), vec![2]);
    assert_eq!(host.read_with(cx, |h, _| h.selected), 2);
    let at = centre(cx, "seg-2");
    cx.simulate_click(at, Modifiers::default());
    assert_eq!(*log.borrow(), vec![2], "the chosen segment stays quiet");
}

#[gpui_kit::test]
fn tab_walks_the_segments_and_enter_or_space_chooses(cx: &mut TestAppContext) {
    let (host, cx, log) = open(cx);
    let step = |cx: &mut VisualTestContext| cx.update(|window, cx| window.focus_next(cx));
    step(cx);
    step(cx);
    cx.simulate_keystrokes("enter");
    assert_eq!(
        *log.borrow(),
        vec![1],
        "the second stop is the second segment"
    );
    step(cx);
    cx.simulate_keystrokes("space");
    assert_eq!(*log.borrow(), vec![1, 2]);
    assert_eq!(host.read_with(cx, |h, _| h.selected), 2);
}

#[gpui_kit::test]
fn fill_and_text_change_over_150ms_and_jump_under_reduce_motion(cx: &mut TestAppContext) {
    motion::clock::freeze();
    let mut motion = Motion::new(false);
    motion.retarget(true, false);
    assert!(motion.chosen.is_running());
    motion::clock::advance(std::time::Duration::from_millis(75));
    let half = motion.chosen.value();
    assert!(half > 0.3 && half < 1., "part way at 75ms: {half}");
    motion::clock::advance(std::time::Duration::from_millis(80));
    assert_eq!(motion.chosen.value(), 1.);
    let mut jump = Motion::new(false);
    jump.retarget(true, true);
    assert_eq!(jump.chosen.value(), 1.);
    let _ = cx;
}

struct Keys;

impl Render for Keys {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        div()
            .p(px(20.))
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(
                Segmented::new(
                    "toggle",
                    [Segment::new("Side by side"), Segment::new("Single view")],
                    0,
                )
                .debug_name("toggle")
                .cap("⌘\\"),
            )
            .child(
                Segmented::new(
                    "picks",
                    [Segment::new("Projects").cap("⌘B"), Segment::new("Session")],
                    0,
                )
                .debug_name("picks"),
            )
    }
}

#[gpui_kit::test]
fn a_toggle_key_is_shown_once_after_the_control_and_a_picking_key_stays_on_its_segment(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
    });
    let (_, cx) = cx.add_window_view(|_, _| Keys);
    cx.run_until_parked();
    let track = cx.debug_bounds("toggle").expect("the track");
    let cap = cx
        .debug_bounds("toggle-cap")
        .expect("the toggle key is drawn once");
    assert!(
        cap.left() >= track.right(),
        "after the control, not inside it"
    );
    assert_eq!(f32::from(cap.left() - track.right()), CAP_GAP);
    let first = cx.debug_bounds("toggle-0").unwrap();
    let second = cx.debug_bounds("toggle-1").unwrap();
    assert_eq!(
        f32::from(first.size.width + second.size.width),
        f32::from(track.size.width) - GAP,
        "no segment holds a cap"
    );
    assert!(
        cx.debug_bounds("picks-cap").is_none(),
        "a segment's own cap adds nothing after the control"
    );
    let plain = cx.debug_bounds("picks-1").unwrap();
    let capped = cx.debug_bounds("picks-0").unwrap();
    assert!(
        capped.size.width > plain.size.width,
        "the segment the key picks carries it"
    );
}
