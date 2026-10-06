use super::*;

#[test]
fn the_first_press_down_lands_on_the_first_option() {
    assert_eq!(step_active(None, 1, 5), Some(0));
}

#[test]
fn the_first_press_up_lands_on_the_last_option() {
    assert_eq!(step_active(None, -1, 5), Some(4));
}

#[test]
fn a_normal_step_moves_by_one() {
    assert_eq!(step_active(Some(2), 1, 5), Some(3));
    assert_eq!(step_active(Some(2), -1, 5), Some(1));
}

#[test]
fn stepping_up_clamps_at_the_first_option() {
    assert_eq!(step_active(Some(0), -1, 5), Some(0));
}

#[test]
fn stepping_down_clamps_at_the_last_option() {
    assert_eq!(step_active(Some(4), 1, 5), Some(4));
}

#[test]
fn an_empty_list_has_nothing_to_highlight() {
    assert_eq!(step_active(None, 1, 0), None);
}

mod clicks {
    use std::{cell::Cell, rc::Rc};

    use gpui_kit::{
        Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled,
        TestAppContext, Window, div, px,
    };

    use crate::{
        select::Select,
        theme::{Appearance, set_appearance},
    };

    /// A select open over a backdrop that counts its own clicks.
    struct Over {
        picked: Rc<Cell<Option<usize>>>,
        backdrop: Rc<Cell<usize>>,
    }

    impl Render for Over {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let (picked, backdrop) = (self.picked.clone(), self.backdrop.clone());
            div()
                .id("backdrop")
                .size_full()
                .on_click(move |_, _, _| backdrop.set(backdrop.get() + 1))
                .child(
                    div().w(px(200.)).child(
                        Select::new("s", ["One", "Two", "Three"])
                            .default_open(true)
                            .on_change(move |i, _, _| picked.set(Some(i))),
                    ),
                )
        }
    }

    /// A pick lands on the option only: the element under the open panel hears nothing.
    #[gpui_kit::test]
    fn a_click_on_an_option_does_not_reach_what_is_under_the_panel(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(true);
        });
        let (picked, backdrop) = (Rc::new(Cell::new(None)), Rc::new(Cell::new(0)));
        let (p, b) = (picked.clone(), backdrop.clone());
        let (_view, cx) = cx.add_window_view(move |_, _| Over { picked: p, backdrop: b });
        cx.run_until_parked();
        let option = cx.debug_bounds("select-option-1").expect("the open panel shows its options");
        cx.simulate_click(option.center(), gpui_kit::Modifiers::default());
        assert_eq!(picked.get(), Some(1), "the option was picked");
        assert_eq!(backdrop.get(), 0, "and the backdrop under the panel heard nothing");
    }

    /// A select at the foot of the window, closed until the test opens it.
    struct Foot;

    impl Render for Foot {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().flex().flex_col().justify_end().p(px(8.)).child(
                div().w(px(200.)).debug_selector(|| "foot-select".into()).child(Select::new("s", ["One", "Two", "Three"])),
            )
        }
    }

    /// With no room below the trigger and room above, the panel opens above it.
    #[gpui_kit::test]
    fn a_select_at_the_foot_of_the_window_opens_upward(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(true);
        });
        let (_view, cx) = cx.add_window_view(|_, _| Foot);
        cx.simulate_resize(gpui_kit::size(px(400.), px(400.)));
        cx.run_until_parked();
        let trigger = cx.debug_bounds("foot-select").expect("the trigger is drawn");
        cx.simulate_click(trigger.center(), gpui_kit::Modifiers::default());
        cx.run_until_parked();
        let option = cx.debug_bounds("select-option-0").expect("the panel shows its options");
        assert!(option.bottom() <= trigger.top(), "the panel is above the trigger: {option:?} over {trigger:?}");
    }
}

mod keys {
    use std::{cell::RefCell, rc::Rc};

    use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

    use crate::{
        select::Select,
        theme::{Appearance, set_appearance},
    };

    /// A select with three options, logging the picks; a second focusable box after it.
    struct Host {
        picks: Rc<RefCell<Vec<usize>>>,
        trigger: gpui_kit::FocusHandle,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let picks = self.picks.clone();
            div().w(px(240.)).p(px(8.)).child(
                Select::new("keys", ["Apple", "Banana", "Cherry"]).selected(Some(1)).focus_handle(&self.trigger).on_change(move |i, _, _| picks.borrow_mut().push(i)),
            )
        }
    }

    fn open_host(cx: &mut TestAppContext) -> (Rc<RefCell<Vec<usize>>>, &mut VisualTestContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            crate::select::bind_keys(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(true);
        });
        let picks = Rc::new(RefCell::new(Vec::new()));
        let seen = picks.clone();
        let (view, cx) = cx.add_window_view(move |_, cx| Host { picks: seen, trigger: cx.focus_handle() });
        cx.run_until_parked();
        // The trigger has focus, as after a click on it or a Tab to it.
        let trigger = view.read_with(cx, |host, _| host.trigger.clone());
        cx.update(|window, cx| trigger.focus(window, cx));
        cx.run_until_parked();
        (picks, cx)
    }

    fn is_open(cx: &mut VisualTestContext) -> bool {
        cx.run_until_parked();
        cx.debug_bounds("select-option-0").is_some()
    }

    #[gpui_kit::test]
    fn down_space_and_return_open_the_list_from_the_trigger(cx: &mut TestAppContext) {
        for key in ["down", "space", "enter"] {
            let (_, cx) = open_host(cx);
            assert!(!is_open(cx), "closed to start with");
            cx.simulate_keystrokes(key);
            assert!(is_open(cx), "{key} opens the list");
        }
    }

    #[gpui_kit::test]
    fn down_and_up_move_the_highlight_and_return_picks_it(cx: &mut TestAppContext) {
        let (picks, cx) = open_host(cx);
        cx.simulate_keystrokes("down");
        assert!(is_open(cx));
        cx.simulate_keystrokes("down enter");
        assert_eq!(*picks.borrow(), [2], "opening lights the chosen Banana; one down is Cherry");
        assert!(!is_open(cx), "a pick closes the list");
    }

    #[gpui_kit::test]
    fn up_goes_back_and_the_ends_stop(cx: &mut TestAppContext) {
        let (picks, cx) = open_host(cx);
        cx.simulate_keystrokes("enter down down down down up enter");
        assert_eq!(*picks.borrow(), [1], "it stops at the last option and one up is the second");
        cx.simulate_keystrokes("enter up up up enter");
        assert_eq!(*picks.borrow(), [1, 0], "and stops at the first");
    }

    #[gpui_kit::test]
    fn home_and_end_go_to_the_first_and_last_option(cx: &mut TestAppContext) {
        let (picks, cx) = open_host(cx);
        cx.simulate_keystrokes("enter end enter");
        assert_eq!(*picks.borrow(), [2]);
        cx.simulate_keystrokes("enter home enter");
        assert_eq!(*picks.borrow(), [2, 0]);
    }

    #[gpui_kit::test]
    fn escape_closes_the_list_and_focus_returns_to_the_trigger(cx: &mut TestAppContext) {
        let (picks, cx) = open_host(cx);
        cx.simulate_keystrokes("enter");
        assert!(is_open(cx));
        cx.simulate_keystrokes("escape");
        assert!(!is_open(cx), "escape closes it");
        assert!(picks.borrow().is_empty(), "and picks nothing");
        cx.simulate_keystrokes("down");
        assert!(is_open(cx), "the trigger has focus again: down opens it");
    }

    #[gpui_kit::test]
    fn typing_the_first_letters_jumps_to_the_option(cx: &mut TestAppContext) {
        let (picks, cx) = open_host(cx);
        cx.simulate_keystrokes("enter c enter");
        assert_eq!(*picks.borrow(), [2], "c goes to Cherry");
        cx.simulate_keystrokes("enter b a enter");
        assert_eq!(*picks.borrow(), [2, 1], "b then a goes to Banana");
    }
}

#[test]
fn type_ahead_finds_the_first_label_that_starts_with_the_letters() {
    let labels: Vec<SharedString> = ["Apple", "Banana", "Blueberry", "Cherry"].into_iter().map(Into::into).collect();
    assert_eq!(type_ahead(&labels, None, "c"), Some(3));
    assert_eq!(type_ahead(&labels, Some(0), "B"), Some(1), "the case does not matter");
    assert_eq!(type_ahead(&labels, Some(1), "bl"), Some(2), "more letters narrow it");
    assert_eq!(type_ahead(&labels, Some(1), "bb"), Some(2), "the same letter again goes to the next option that starts with it");
    assert_eq!(type_ahead(&labels, Some(2), "bbb"), Some(1), "and wraps");
    assert_eq!(type_ahead(&labels, None, "z"), None);
    assert_eq!(type_ahead(&labels, None, ""), None);
    assert_eq!(type_ahead(&[], None, "a"), None);
}

mod marks {
    use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px};

    use crate::{
        model_badge::BrandMark,
        select::{Select, SelectOption},
        theme::{Appearance, set_appearance},
    };

    struct Marked(bool);

    impl Render for Marked {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let first = SelectOption::from("Claude");
            let first = if self.0 { first.mark(BrandMark::new("a.svg", "a.svg")) } else { first };
            div().w(px(240.)).child(Select::new("m", [first, SelectOption::from("Atelier")]).default_open(true))
        }
    }

    fn drawn(marked: bool, cx: &mut TestAppContext) -> (bool, bool) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(true);
        });
        let (_, cx) = cx.add_window_view(move |_, _| Marked(marked));
        cx.run_until_parked();
        (cx.debug_bounds("select-monogram-A").is_some(), cx.debug_bounds("select-monogram-C").is_some())
    }

    #[gpui_kit::test]
    fn an_option_without_a_mark_shows_its_letter_where_the_mark_goes(cx: &mut TestAppContext) {
        assert_eq!(drawn(true, cx), (true, false), "Atelier has no mark beside Claude, which has one");
    }

    #[gpui_kit::test]
    fn a_list_with_no_marks_at_all_stays_plain(cx: &mut TestAppContext) {
        assert_eq!(drawn(false, cx), (false, false));
    }
}

mod dismiss {
    use std::{cell::Cell, rc::Rc};

    use gpui_kit::{
        Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, TestAppContext,
        VisualTestContext, Window, div, point, px,
    };

    use crate::{
        select::Select,
        theme::{Appearance, set_appearance},
    };

    /// A select and, under the whole window, an element that counts the clicks and hovers it gets.
    struct Page {
        clicks: Rc<Cell<usize>>,
        hovers: Rc<Cell<usize>>,
    }

    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let (clicks, hovers) = (self.clicks.clone(), self.hovers.clone());
            div()
                .id("page")
                .size_full()
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                .on_hover(move |on, _, _| {
                    if *on {
                        hovers.set(hovers.get() + 1)
                    }
                })
                .child(div().w(px(200.)).m(px(8.)).debug_selector(|| "the-select".into()).child(Select::new("s", ["One", "Two", "Three"])))
        }
    }

    fn page(cx: &mut TestAppContext) -> (Rc<Cell<usize>>, Rc<Cell<usize>>, &mut VisualTestContext) {
        page_with(true, cx)
    }

    /// The same page, with the motion on or off.
    fn page_with(reduce: bool, cx: &mut TestAppContext) -> (Rc<Cell<usize>>, Rc<Cell<usize>>, &mut VisualTestContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            crate::select::bind_keys(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(reduce);
        });
        let (clicks, hovers) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
        let (c, h) = (clicks.clone(), hovers.clone());
        let (_view, cx) = cx.add_window_view(move |_, _| Page { clicks: c, hovers: h });
        cx.simulate_resize(gpui_kit::size(px(400.), px(400.)));
        cx.run_until_parked();
        (clicks, hovers, cx)
    }

    fn open(cx: &mut VisualTestContext) {
        let at = cx.debug_bounds("the-select").unwrap().center();
        cx.simulate_click(at, gpui_kit::Modifiers::default());
        cx.run_until_parked();
        assert!(cx.debug_bounds("select-option-0").is_some(), "the list is open");
    }

    #[gpui_kit::test]
    fn a_click_outside_closes_the_list_and_the_page_under_it_hears_nothing(cx: &mut TestAppContext) {
        let (clicks, _, cx) = page(cx);
        open(cx);
        let before = clicks.get();
        cx.simulate_click(point(px(350.), px(350.)), gpui_kit::Modifiers::default());
        cx.run_until_parked();
        assert!(cx.debug_bounds("select-option-0").is_none(), "it closed");
        assert_eq!(clicks.get(), before, "and the click went no further");
    }

    #[gpui_kit::test]
    fn the_page_is_not_hovered_while_the_list_is_open(cx: &mut TestAppContext) {
        let (_, hovers, cx) = page(cx);
        cx.simulate_mouse_move(point(px(390.), px(390.)), None, gpui_kit::Modifiers::default());
        cx.run_until_parked();
        let before = hovers.get();
        open(cx);
        cx.simulate_mouse_move(point(px(300.), px(300.)), None, gpui_kit::Modifiers::default());
        cx.simulate_mouse_move(point(px(350.), px(350.)), None, gpui_kit::Modifiers::default());
        cx.run_until_parked();
        assert_eq!(hovers.get(), before, "the backdrop blocks hover");
    }

    /// Lets the animations run out: frames of 16 ms on the frozen clock, each drawn.
    fn settle(cx: &mut VisualTestContext) {
        crate::motion::clock::freeze();
        for _ in 0..60 {
            crate::motion::clock::advance(std::time::Duration::from_millis(16));
            cx.update(|window, _| window.refresh());
            cx.run_until_parked();
        }
    }

    /// With the motion on: a press opens the list, a press on the trigger shuts it, and the next press opens it
    /// again at once, the first time, with the trigger answering the pointer in between.
    #[gpui_kit::test]
    fn a_press_after_a_close_opens_the_list_the_first_time(cx: &mut TestAppContext) {
        let (_, _, cx) = page_with(false, cx);
        let at = cx.debug_bounds("the-select").unwrap().center();
        cx.simulate_click(at, gpui_kit::Modifiers::default());
        settle(cx);
        assert!(cx.debug_bounds("select-option-0").is_some(), "the first press opens it");
        cx.simulate_click(at, gpui_kit::Modifiers::default());
        settle(cx);
        assert!(cx.debug_bounds("select-option-0").is_none(), "the second press shuts it");
        cx.simulate_mouse_move(point(px(390.), px(390.)), None, gpui_kit::Modifiers::default());
        cx.simulate_mouse_move(at, None, gpui_kit::Modifiers::default());
        settle(cx);
        cx.simulate_click(at, gpui_kit::Modifiers::default());
        settle(cx);
        assert!(cx.debug_bounds("select-option-0").is_some(), "the next press opens it the first time");
    }

    /// Frames of 16 ms for `ms` on the frozen clock.
    fn run_for(ms: u64, cx: &mut VisualTestContext) {
        crate::motion::clock::freeze();
        for _ in 0..(ms / 16).max(1) {
            crate::motion::clock::advance(std::time::Duration::from_millis(16));
            cx.update(|window, _| window.refresh());
            cx.run_until_parked();
        }
    }

    /// The press comes while the list is still coming in, and the pointer stays on the trigger throughout.
    #[gpui_kit::test]
    fn pressing_twice_quickly_then_again_still_opens_the_first_time(cx: &mut TestAppContext) {
        let (_, _, cx) = page_with(false, cx);
        let at = cx.debug_bounds("the-select").unwrap().center();
        cx.simulate_mouse_move(at, None, gpui_kit::Modifiers::default());
        run_for(50, cx);
        cx.simulate_click(at, gpui_kit::Modifiers::default());
        run_for(120, cx);
        cx.simulate_click(at, gpui_kit::Modifiers::default());
        run_for(600, cx);
        assert!(cx.debug_bounds("select-option-0").is_none(), "the second press shut it");
        cx.simulate_click(at, gpui_kit::Modifiers::default());
        run_for(600, cx);
        assert!(cx.debug_bounds("select-option-0").is_some(), "the third press opens it the first time");
    }

    #[gpui_kit::test]
    fn a_click_on_the_trigger_of_an_open_list_closes_it(cx: &mut TestAppContext) {
        let (clicks, _, cx) = page(cx);
        open(cx);
        let before = clicks.get();
        let at = cx.debug_bounds("the-select").unwrap().center();
        cx.simulate_click(at, gpui_kit::Modifiers::default());
        cx.run_until_parked();
        assert!(cx.debug_bounds("select-option-0").is_none(), "the trigger toggles it shut");
        assert_eq!(clicks.get(), before, "and the page heard nothing of it");
    }
}

/// Rows are 28 tall with a 2px gap, in the panel with its 4px padding and 1px edge; a heading is one more row.
#[test]
fn the_panel_is_rows_of_28_with_2px_between_and_an_edge() {
    assert_eq!(panel_height_of(3, 0), 2. * 4. + 3. * 28. + 2. * 2. + 2.);
    assert_eq!(panel_height_of(3, 1), 2. * 4. + 3. * 28. + 26. + 3. * 2. + 2.);
    assert_eq!((ITEM_HEIGHT, ROW_GAP), (28., 2.));
}
/// The options come in as the web's MorphSelect has them: 80ms after the open, 35ms apart, 0.3s each.
#[test]
fn the_options_come_in_on_the_webs_stagger() {
    assert_eq!(opens_in(0), 0.);
    assert!((opens_in(1) - 0.38).abs() < 1e-6);
    assert!((opens_in(4) - (0.08 + 0.035 * 3. + 0.3)).abs() < 1e-6);
}
/// The option's rise is Motion's spring for `y` (stiffness 500, damping 25): it starts at rest, overshoots a
/// little, and settles on 1.
#[test]
fn the_rise_spring_starts_at_rest_overshoots_and_settles() {
    assert_eq!(spring_unit(500., 25., 0.), 0.);
    assert!(spring_unit(500., 25., 0.01) < 0.05);
    let peak = (0..200).map(|i| spring_unit(500., 25., i as f32 * 0.005)).fold(0., f32::max);
    // zeta = 25 / (2 sqrt 500) = 0.559, so the first overshoot is exp(-pi zeta / sqrt(1 - zeta^2)) = 12%.
    assert!((peak - 1.12).abs() < 0.01, "{peak}");
    assert!((spring_unit(500., 25., 1.) - 1.).abs() < 1e-3);
    // The spring and the closed form agree with what `Animated` integrates.
    let mut spring = crate::motion::Animated::new(crate::motion::Spring { stiffness: 500., damping: 25., mass: 1. }, 0.);
    spring.set_target(1.);
    for _ in 0..10 {
        spring.step(0.01, false);
    }
    assert!((spring.value() - spring_unit(500., 25., 0.1)).abs() < 0.01, "{} vs {}", spring.value(), spring_unit(500., 25., 0.1));
}
/// The surface is the trigger at 0 and the panel at 1, and its height is the trigger plus the list.
#[test]
fn the_surface_is_the_trigger_then_the_panel() {
    assert_eq!(surface_at((80., 32.), 208., 130., 0.), (80., 32.));
    assert_eq!(surface_at((80., 32.), 208., 130., 1.), (208., 162.));
    let (w, h) = surface_at((80., 32.), 208., 130., 0.5);
    assert_eq!((w, h), (144., 97.));
    assert_eq!(list_height_of(3, 0), 2. * 4. + 3. * 28. + 2. * 2.);
}
/// A trigger lights for a pointer that has not moved only when nothing covers it and it is inside.
#[test]
fn a_still_pointer_lights_an_uncovered_trigger_it_is_inside() {
    use gpui_kit::{Bounds, point, px, size};
    let trigger = Some(Bounds::new(point(px(10.), px(10.)), size(px(100.), px(30.))));
    let inside = point(px(50.), px(20.));
    let outside = point(px(300.), px(300.));
    assert!(should_light(false, false, trigger, inside), "uncovered, dull, inside: light it");
    assert!(!should_light(true, false, trigger, inside), "the list covers it");
    assert!(!should_light(false, true, trigger, inside), "already lit");
    assert!(!should_light(false, false, trigger, outside), "the pointer is elsewhere");
    assert!(!should_light(false, false, None, inside), "not measured yet");
}

/// The web's morph spring, `{ duration: 0.5, bounce: 0.22 }`, run twice as fast: `duration: 0.25`.
#[test]
fn the_morph_is_the_webs_spring_at_twice_the_speed() {
    let spring = crate::motion::Spring::select_morph();
    assert_eq!(crate::motion::SELECT_MORPH_SECONDS, 0.25);
    assert!((spring.stiffness - (std::f32::consts::TAU / 0.25).powi(2)).abs() < 1e-2);
    assert!((spring.damping - 2. * 0.78 * spring.stiffness.sqrt()).abs() < 1e-3);
}
/// A select trigger tints toward the Ghost button's hover on the pointer and while open.
#[test]
fn the_trigger_tone_is_the_ghost_hover_for_every_trigger() {
    for theme in crate::themes::all() {
        let ghost = crate::button::colors(crate::button::ButtonVariant::Ghost, theme, 1., false).0;
        let clear = gpui_kit::transparent_black();
        assert_eq!(trigger_tone(theme, clear, 0.), clear);
        assert_eq!(trigger_tone(theme, clear, 1.), ghost, "{}: a chip is the Ghost hover", theme.name);
        assert!((trigger_tone(theme, theme.card, 0.).l - theme.card.l).abs() < 1e-5);
        let held = trigger_tone(theme, theme.card, 1.);
        assert_ne!(held, theme.card, "{}: a field steps too", theme.name);
        let want = crate::theme::mix(theme.card, theme.foreground, 0.06);
        assert!((held.l - want.l).abs() < 1e-5 && (held.s - want.s).abs() < 1e-5);
    }
}

/// At 60 fps the morph's progress follows the web's spring frame by frame: Motion's `{ duration: 0.5, bounce: 0.22 }`
/// is this damped spring, and the surface is the trigger at 0 and the panel at 1.
#[test]
fn the_morph_follows_the_webs_spring_at_sixty_frames_a_second() {
    let spring = crate::motion::Spring::select_morph();
    let mut morph = crate::motion::Animated::new(spring, 0.);
    morph.set_target(1.);
    let mut worst: f32 = 0.;
    let mut peak: f32 = 0.;
    for frame in 1..=60 {
        morph.step(1. / 60., false);
        let t = frame as f32 / 60.;
        let web = spring_unit(spring.stiffness, spring.damping, t);
        worst = worst.max((morph.value() - web).abs());
        peak = peak.max(morph.value());
    }
    assert!(worst < 0.01, "worst frame differs by {worst}");
    // zeta = 0.78, so the surface goes over its size by exp(-pi zeta / sqrt(1 - zeta^2)) = 2%, then settles.
    assert!((peak - 1.02).abs() < 0.005, "{peak}");
    assert!((morph.value() - 1.).abs() < 0.01);
}

mod morph {
    use gpui_kit::{Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px};
    use crate::{
        select::{Select, list_height_of},
        theme::{Appearance, set_appearance},
    };
    struct Page {
        down: bool,
    }
    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let select = Select::new("m", ["One", "Two", "Three"]).selected(Some(1)).default_open(true);
            let frame = div().w(px(200.)).debug_selector(|| "the-select".into()).child(select);
            if self.down {
                div().size_full().p(px(8.)).child(frame)
            } else {
                div().size_full().flex().flex_col().justify_end().p(px(8.)).child(frame)
            }
        }
    }
    fn open(down: bool, cx: &mut TestAppContext) -> &mut gpui_kit::VisualTestContext {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(true);
        });
        let (_view, cx) = cx.add_window_view(move |_, _| Page { down });
        cx.simulate_resize(gpui_kit::size(px(400.), px(400.)));
        cx.run_until_parked();
        cx
    }
    /// Under Reduce Motion the open surface is at once the panel: the trigger's top and left, and the trigger's height plus the list.
    #[gpui_kit::test]
    fn the_surface_grows_from_the_trigger_down(cx: &mut TestAppContext) {
        let cx = open(true, cx);
        let trigger = cx.debug_bounds("the-select").unwrap();
        let surface = cx.debug_bounds("select-surface").expect("the surface is drawn");
        assert_eq!((surface.left(), surface.top()), (trigger.left(), trigger.top()), "it starts on the trigger");
        let header = cx.debug_bounds("select-header").expect("the header is drawn");
        assert_eq!((header.left(), header.top(), header.size.height), (trigger.left(), trigger.top(), trigger.size.height), "the header is the trigger's row");
        assert_eq!(f32::from(surface.size.height), f32::from(trigger.size.height) + list_height_of(3, 0));
        assert_eq!(surface.size.width, trigger.size.width);
        let first = cx.debug_bounds("select-option-0").unwrap();
        assert!(first.top() >= header.bottom(), "the options are under the header");
        let face = cx.debug_bounds("select-header-face").unwrap();
        assert_eq!(f32::from(face.left() - surface.left()), 14., "the header lines up with the options");
    }
    /// At the foot of the window it grows up from the trigger: the trigger's bottom stays put.
    #[gpui_kit::test]
    fn the_surface_grows_from_the_trigger_up(cx: &mut TestAppContext) {
        let cx = open(false, cx);
        let trigger = cx.debug_bounds("the-select").unwrap();
        let surface = cx.debug_bounds("select-surface").expect("the surface is drawn");
        assert_eq!(surface.bottom(), trigger.bottom(), "its foot stays on the trigger's");
        let header = cx.debug_bounds("select-header").unwrap();
        assert_eq!(header.bottom(), trigger.bottom());
        let last = cx.debug_bounds("select-option-2").unwrap();
        assert!(last.bottom() <= header.top(), "the options are over the header");
    }
}

#[test]
fn the_header_inset_eases_from_the_trigger_to_the_options() {
    assert_eq!(header_inset(12., 0.), 12.);
    assert_eq!(header_inset(8., 1.), 14.);
    assert_eq!(header_inset(12., 0.5), 13.);
    assert_eq!(header_inset(12., 2.), 14., "the overshoot does not push it past");
}

#[test]
fn a_select_rounds_every_corner_until_it_is_told_to_join_a_row() {
    let all = Select::new("s", ["a"]).corners;
    assert!(all.top_left && all.top_right && all.bottom_left && all.bottom_right);
    let corners = crate::button_group::segment_corners(0, 2, gpui_kit::Axis::Horizontal);
    let first = Select::new("s", ["a"]).corners(corners).corners;
    assert!(first.top_left && first.bottom_left && !first.top_right && !first.bottom_right);
}

#[test]
fn a_select_has_no_fill_of_its_own_until_given_one() {
    assert!(Select::new("s", ["a"]).fill.is_none());
    assert!(Select::new("s", ["a"]).fill(gpui_kit::hsla(0., 0., 0.5, 1.)).fill.is_some());
}

#[test]
fn the_panel_is_square_where_the_trigger_is_square_and_round_at_its_far_end() {
    use gpui_kit::Corners;
    // The right part of two: its left side meets a neighbour.
    let right_part = Corners { top_left: false, top_right: true, bottom_left: false, bottom_right: true };
    let down = super::helpers::panel_corners(right_part, false);
    assert_eq!((down.top_left, down.top_right), (false, true), "the top follows the trigger");
    assert_eq!((down.bottom_left, down.bottom_right), (true, true), "the far end is round");
    let up = super::helpers::panel_corners(right_part, true);
    assert_eq!((up.bottom_left, up.bottom_right), (false, true), "opening up, the bottom follows it");
    assert_eq!((up.top_left, up.top_right), (true, true));
    let alone = Corners { top_left: true, top_right: true, bottom_left: true, bottom_right: true };
    assert_eq!(super::helpers::panel_corners(alone, false), alone, "a lone select is round all round");
}
