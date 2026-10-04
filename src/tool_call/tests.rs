use super::*;

#[test]
fn a_run_that_starts_running_opens() {
    assert!(should_open(ToolStatus::Running, false, false, false));
}

#[test]
fn output_arriving_on_an_already_running_call_opens_it() {
    // The bug: a Running call with no output yet, then output shows up while it is still running.
    assert!(should_open(ToolStatus::Running, true, false, true));
}

#[test]
fn output_that_was_already_there_does_not_reopen_it() {
    assert!(!should_open(ToolStatus::Running, true, true, true));
}

#[test]
fn a_call_with_no_output_at_all_never_opens() {
    assert!(!should_open(ToolStatus::Running, true, false, false));
}

#[test]
fn a_finished_call_does_not_open_just_because_it_has_a_body() {
    assert!(!should_open(ToolStatus::Done, false, false, true));
}

mod folding {
    use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};

    use super::super::*;
    use crate::theme::{Appearance, set_appearance};

    struct Host {
        status: ToolStatus,
        keep_open: bool,
        starts_open: bool,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let call = ToolCall::new("call", "Ran tests").tool("cargo test").status(self.status).output("3 passed");
            let call = if self.keep_open { call.collapse_on_complete(false) } else { call };
            div().w(px(600.)).child(if self.starts_open { call.default_open(true) } else { call })
        }
    }

    fn settle(host: &gpui_kit::Entity<Host>, cx: &mut gpui_kit::VisualTestContext) {
        for _ in 0..4 {
            cx.run_until_parked();
            host.update(cx, |_, cx| cx.notify());
        }
        cx.run_until_parked();
    }

    fn open(status: ToolStatus, keep_open: bool, starts_open: bool, cx: &mut TestAppContext) -> (gpui_kit::Entity<Host>, &mut gpui_kit::VisualTestContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Light, cx);
            cx.set_reduce_motion(true);
        });
        let (host, cx) = cx.add_window_view(move |_, _| Host { status, keep_open, starts_open });
        cx.simulate_resize(size(px(700.), px(500.)));
        settle(&host, cx);
        (host, cx)
    }

    fn finish(host: &gpui_kit::Entity<Host>, cx: &mut gpui_kit::VisualTestContext) {
        host.update(cx, |h, cx| {
            h.status = ToolStatus::Done;
            cx.notify();
        });
        settle(host, cx);
    }

    #[gpui_kit::test]
    fn a_call_closes_itself_when_it_finishes_by_default(cx: &mut TestAppContext) {
        let (host, cx) = open(ToolStatus::Running, false, false, cx);
        assert!(cx.debug_bounds("tool-output").is_some(), "it is open while it runs");
        finish(&host, cx);
        assert!(cx.debug_bounds("tool-output").is_none());
    }

    #[gpui_kit::test]
    fn a_call_told_not_to_collapse_stays_open_when_it_finishes(cx: &mut TestAppContext) {
        let (host, cx) = open(ToolStatus::Running, true, false, cx);
        finish(&host, cx);
        assert!(cx.debug_bounds("tool-output").is_some());
    }

    #[gpui_kit::test]
    fn a_finished_call_opens_at_once_when_asked_to(cx: &mut TestAppContext) {
        let (_, cx) = open(ToolStatus::Done, true, true, cx);
        assert!(cx.debug_bounds("tool-output").is_some());
    }
}

mod clipped {
    use std::{cell::Cell, rc::Rc};

    use gpui_kit::{Context, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};

    use super::super::*;
    use crate::theme::{Appearance, set_appearance};

    struct Host {
        lines: usize,
        opened: Rc<Cell<usize>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let log = (0..self.lines).map(|n| format!("line {n}")).collect::<Vec<_>>().join("\n");
            let opened = self.opened.clone();
            div().w(px(600.)).child(
                ToolCall::new("call", "Ran tests")
                    .status(ToolStatus::Done)
                    .output(log)
                    .default_open(true)
                    .collapse_on_complete(false)
                    .preview_rows(6)
                    .on_open(move |_, _| opened.set(opened.get() + 1)),
            )
        }
    }

    fn open(lines: usize, cx: &mut TestAppContext) -> (gpui_kit::Entity<Host>, &mut gpui_kit::VisualTestContext, Rc<Cell<usize>>) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Light, cx);
            cx.set_reduce_motion(true);
        });
        let opened = Rc::new(Cell::new(0));
        let seen = opened.clone();
        let (host, cx) = cx.add_window_view(move |_, _| Host { lines, opened });
        cx.simulate_resize(size(px(700.), px(900.)));
        settle(&host, cx);
        (host, cx, seen)
    }

    fn settle(host: &gpui_kit::Entity<Host>, cx: &mut gpui_kit::VisualTestContext) {
        for _ in 0..4 {
            cx.run_until_parked();
            host.update(cx, |_, cx| cx.notify());
        }
        cx.run_until_parked();
    }

    fn height(cx: &mut gpui_kit::VisualTestContext) -> f32 {
        f32::from(cx.debug_bounds("tool-output").unwrap().size.height)
    }

    fn press(cx: &mut gpui_kit::VisualTestContext) {
        let at = cx.debug_bounds("tool-output").unwrap().center();
        cx.simulate_click(at, Modifiers::none());
    }

    #[gpui_kit::test]
    fn a_long_log_shows_only_its_last_lines(cx: &mut TestAppContext) {
        let (_, cx, opened) = open(40, cx);
        assert_eq!(height(cx), 6. * 20. + 24.);
        assert_eq!(opened.get(), 0);
    }

    #[gpui_kit::test]
    fn a_short_log_is_as_tall_as_its_lines(cx: &mut TestAppContext) {
        let (_, cx, _) = open(2, cx);
        assert!(height(cx) < 6. * 20. + 24.);
    }

    #[gpui_kit::test]
    fn pressing_it_opens_it_and_a_second_press_folds_it(cx: &mut TestAppContext) {
        let (host, cx, opened) = open(40, cx);
        let clipped = height(cx);
        press(cx);
        settle(&host, cx);
        assert_eq!(opened.get(), 1);
        assert!(height(cx) > clipped + 100., "it grew: {} from {clipped}", height(cx));
        press(cx);
        settle(&host, cx);
        assert_eq!(opened.get(), 1, "folding is silent");
        assert_eq!(height(cx), clipped);
    }
}

mod motion {
    use std::time::Duration;

    use gpui_kit::{Context, InteractiveElement, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, Window, div, point, px, size};

    use super::super::*;
    use crate::theme::{Appearance, set_appearance};

    struct Host;

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let log = (0..6).map(|n| format!("line {n}")).collect::<Vec<_>>().join("\n");
            div()
                .w(px(600.))
                .flex()
                .flex_col()
                .child(ToolCall::new("call", "Ran tests").status(ToolStatus::Done).output(log).default_open(true).collapse_on_complete(false))
                .child(div().debug_selector(|| "below".into()).h(px(10.)))
        }
    }

    /// Where the row under the call sits, frame by frame, while the call opens or closes.
    fn below_over_time(cx: &mut gpui_kit::VisualTestContext, host: &gpui_kit::Entity<Host>) -> Vec<f32> {
        (0..30)
            .map(|_| {
                crate::motion::clock::advance(Duration::from_millis(16));
                host.update(cx, |_, cx| cx.notify());
                cx.run_until_parked();
                f32::from(cx.debug_bounds("below").unwrap().top())
            })
            .collect()
    }

    #[gpui_kit::test]
    fn what_is_under_a_call_glides_as_it_closes_and_opens(cx: &mut TestAppContext) {
        crate::motion::clock::freeze();
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Light, cx);
        });
        let (host, cx) = cx.add_window_view(|_, _| Host);
        cx.simulate_resize(size(px(700.), px(900.)));
        let open_at = below_over_time(cx, &host).last().copied().unwrap();
        cx.simulate_click(point(px(300.), px(10.)), Modifiers::none());
        let closing = below_over_time(cx, &host);
        let closed_at = *closing.last().unwrap();
        assert!(closed_at < open_at - 50., "it closed: {open_at} to {closed_at}");
        let between = |path: &[f32], from: f32, to: f32| path.iter().filter(|y| **y < from.max(to) - 1. && **y > from.min(to) + 1.).count();
        assert!(between(&closing, open_at, closed_at) >= 4, "the row under it moves through the close, not at its end: {closing:?}");
        assert!(closing.windows(2).all(|w| w[1] <= w[0] + 0.5), "and only up: {closing:?}");
        cx.simulate_click(point(px(300.), px(10.)), Modifiers::none());
        let opening = below_over_time(cx, &host);
        assert!((opening.last().unwrap() - open_at).abs() < 0.5, "it opened back to where it was: {opening:?}");
        assert!(between(&opening, closed_at, open_at) >= 4, "the row under it moves through the open, not at its start: {opening:?}");
        assert!(opening.windows(2).all(|w| w[1] >= w[0] - 0.5), "and only down: {opening:?}");
    }
}
