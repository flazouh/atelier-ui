use super::*;

#[test]
fn the_steps_take_things_away_in_order() {
    let off = |f: fn(&Step) -> bool| STEPS.iter().position(|s| !f(s)).unwrap();
    assert_eq!(off(|s| s.line), 1);
    assert_eq!(off(|s| s.totals), 2);
    assert_eq!(off(|s| s.nav_words), 3);
    assert_eq!(STEPS.iter().position(|s| s.short_summary), Some(4), "the count shortens after the words go");
    assert_eq!(STEPS.iter().position(|s| s.short_scopes), Some(5), "the scope words shorten after the count");
    assert_eq!(STEPS.iter().position(|s| !s.scope_cap), Some(6), "the scope key cap goes last");
    assert_eq!(STEPS.iter().position(|s| s.count_only), Some(7), "the word after the count goes last of all");
    // Once gone, a thing stays gone at every narrower step.
    for w in STEPS.windows(2) {
        let (a, b) = (w[0], w[1]);
        assert!(a.line >= b.line && a.totals >= b.totals && a.nav_words >= b.nav_words && a.scope_cap >= b.scope_cap && a.count_only <= b.count_only && a.short_scopes <= b.short_scopes && a.short_summary <= b.short_summary);
    }
}

#[test]
fn the_first_step_that_fits_is_shown() {
    let needed = [Some(900.), Some(850.), Some(780.), Some(600.), Some(520.), Some(480.)];
    assert_eq!(choose(1000., &needed), 0);
    assert_eq!(choose(800., &needed), 2);
    assert_eq!(choose(100., &needed), 5, "the narrowest when nothing fits");
}

#[test]
fn a_step_not_measured_yet_is_tried_before_a_narrower_one() {
    assert_eq!(choose(800., &[Some(900.), None, None, None, None, None]), 1);
    assert_eq!(choose(1000., &[None; 6]), 0);
}

#[test]
fn a_step_whose_count_does_not_fit_its_box_needs_more_than_the_bar_has() {
    // The parts add up to 480 in a bar of 500, but the count is 60 wide in a box of 40: it clips, so the step is out.
    assert_eq!(need_at(480., 500., 60., 40.), 501.);
    // Fitting, the sum stands, with the pixel or two that layout rounding takes.
    assert_eq!(need_at(480., 500., 60., 60.), 480. + FIT_SLACK);
    assert_eq!(need_at(480., 500., 60.4, 60.), 480. + FIT_SLACK, "half a pixel is rounding, not a clip");
    // A step that needs more than the width anyway keeps its own, larger need.
    assert_eq!(need_at(900., 500., 60., 40.), 900. + FIT_SLACK);
}

#[test]
fn the_menu_holds_what_the_owner_handles_in_order() {
    let words = |h: &ReviewHandlers| menu_entries(h).into_iter().map(|(w, _, _)| w).collect::<Vec<_>>();
    assert!(words(&ReviewHandlers::default()).is_empty());
    let h = ReviewHandlers::default().on_put_back(|_, _| {}).on_reject_all(|_, _| {}).on_accept_all(|_, _| {});
    assert_eq!(words(&h), ["Accept all", "Reject all", "Put all back"]);
    // Every decision over all files shows its key.
    assert!(menu_entries(&h).iter().take(2).all(|(_, cap, _)| cap.is_some()));
}

mod fit {
    use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};

    use crate::{
        review::{ReviewHandlers, ReviewProgress},
        review_bar::ReviewBar,
        theme::{Appearance, set_appearance},
    };

    struct Bar {
        files: usize,
        reviewed: usize,
        scoped: bool,
        width: f32,
    }

    impl Render for Bar {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let mut handlers = ReviewHandlers::default().on_mark(|_, _| {}).on_review_mode(|_, _| {}).on_previous(|_, _| {}).on_next(|_, _| {});
            handlers.on_switch_scope = Some(std::rc::Rc::new(|_, _| {}));
            let bar = ReviewBar::new("bar", ReviewProgress { files: self.files, reviewed: self.reviewed, added: 1200, removed: 340 }, handlers)
                .reviewed_word("reviewed");
            let bar = if self.scoped { bar.scopes([("This turn".into(), "Turn".into()), ("Whole session".into(), "Session".into())], 0) } else { bar };
            div().w(px(self.width)).child(bar)
        }
    }

    /// Whether the summary shows whole in a bar `width` wide, after the bar has settled on a step.
    fn fits(files: usize, width: f32, cx: &mut TestAppContext) -> (f32, f32) {
        fits_with(files, 1, false, width, cx)
    }

    fn fits_with(files: usize, reviewed: usize, scoped: bool, width: f32, cx: &mut TestAppContext) -> (f32, f32) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(move |_, _| Bar { files, reviewed, scoped, width });
        cx.simulate_resize(size(px(1200.), px(200.)));
        for _ in 0..12 {
            cx.run_until_parked();
            view.update(cx, |_, cx| cx.notify());
        }
        cx.run_until_parked();
        let inner = cx.debug_bounds("review-bar-summary").expect("the summary is drawn").size.width;
        let outer = cx.debug_bounds("review-bar-summary-box").expect("its box is drawn").size.width;
        (f32::from(inner), f32::from(outer))
    }

    #[gpui_kit::test]
    fn a_254_file_count_fits_the_bar_of_a_narrow_pane(cx: &mut TestAppContext) {
        for width in [420., 460., 520.] {
            let (inner, outer) = fits(254, width, cx);
            assert!(inner <= outer + 0.5, "at {width}px the summary is {inner}px in a box of {outer}px");
        }
    }

    #[gpui_kit::test]
    fn the_count_never_clips_in_a_bar_with_two_scopes_from_520px_up(cx: &mut TestAppContext) {
        for (files, reviewed) in [(1, 1), (1, 0), (12, 12), (254, 3)] {
            for width in (520..=900).step_by(40) {
                let (inner, outer) = fits_with(files, reviewed, true, width as f32, cx);
                assert!(inner <= outer + 0.5, "{reviewed}/{files} at {width}px: the summary is {inner}px in a box of {outer}px");
            }
        }
    }
}

mod menu_keys {
    use std::{cell::RefCell, rc::Rc};

    use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};

    use crate::{
        review::{ReviewHandlers, ReviewProgress},
        review_bar::ReviewBar,
        theme::{Appearance, set_appearance},
    };

    struct Page {
        log: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let (a, b) = (self.log.clone(), self.log.clone());
            let handlers = ReviewHandlers::default().on_accept_all(move |_, _| a.borrow_mut().push("accept all")).on_reject_all(move |_, _| b.borrow_mut().push("reject all"));
            div().w(px(900.)).child(ReviewBar::new("bar", ReviewProgress { files: 3, reviewed: 1, added: 4, removed: 2 }, handlers))
        }
    }

    fn open(cx: &mut TestAppContext) -> (Rc<RefCell<Vec<&'static str>>>, &mut gpui_kit::VisualTestContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Light, cx);
            cx.set_reduce_motion(true);
        });
        let log = Rc::new(RefCell::new(Vec::new()));
        let shared = log.clone();
        let (page, cx) = cx.add_window_view(move |_, _| Page { log: shared });
        cx.simulate_resize(size(px(1000.), px(400.)));
        for _ in 0..3 {
            cx.run_until_parked();
            page.update(cx, |_, cx| cx.notify());
        }
        let at = cx.debug_bounds("review-bar-more").expect("the menu button is drawn").center();
        cx.simulate_click(at, gpui_kit::Modifiers::default());
        for _ in 0..4 {
            cx.run_until_parked();
            page.update(cx, |_, cx| cx.notify());
        }
        (log, cx)
    }

    #[gpui_kit::test]
    fn the_menu_opens_on_its_first_entry_and_enter_runs_it(cx: &mut TestAppContext) {
        let (log, cx) = open(cx);
        cx.simulate_keystrokes("enter");
        cx.run_until_parked();
        assert_eq!(*log.borrow(), ["accept all"], "the first entry had focus");
    }

    #[gpui_kit::test]
    fn the_down_arrow_reaches_the_next_entry_and_enter_runs_that_one(cx: &mut TestAppContext) {
        let (log, cx) = open(cx);
        cx.simulate_keystrokes("down");
        cx.simulate_keystrokes("enter");
        cx.run_until_parked();
        assert_eq!(*log.borrow(), ["reject all"]);
    }

    #[gpui_kit::test]
    fn the_up_arrow_goes_back_and_space_runs_an_entry_too(cx: &mut TestAppContext) {
        let (log, cx) = open(cx);
        cx.simulate_keystrokes("down");
        cx.simulate_keystrokes("up");
        cx.simulate_keystrokes("space");
        cx.run_until_parked();
        assert_eq!(*log.borrow(), ["accept all"]);
    }
}
