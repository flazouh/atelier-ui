use gpui_kit::{Hsla, Rgba};

use super::*;
use crate::{
    theme::{can_be_primary, with_pick},
    themes,
};

fn blue() -> Hsla {
    Rgba { r: 2. / 255., g: 133. / 255., b: 247. / 255., a: 1. }.into()
}

#[test]
fn by_default_the_primary_button_is_the_page_inverted_in_every_theme() {
    for theme in themes::all() {
        assert_eq!(colors(ButtonVariant::Primary, theme, 0., true), (theme.foreground, theme.background), "{}", theme.name);
    }
}

#[test]
fn a_picked_colour_fills_the_primary_button_in_every_theme_and_no_other_button() {
    for theme in themes::all() {
        let shown = with_pick(theme, Some(blue()));
        if !can_be_primary(theme, blue()) {
            continue;
        }
        let (fill, text) = colors(ButtonVariant::Primary, &shown, 0., true);
        assert_eq!(fill, blue(), "{}: the fill is the pick", theme.name);
        assert!(text == theme.background || text == theme.foreground, "{}: the text is the page or the ink", theme.name);
        for other in [ButtonVariant::Secondary, ButtonVariant::Ghost, ButtonVariant::Invert] {
            assert_eq!(colors(other, &shown, 0.5, false), colors(other, theme, 0.5, false), "{}: {other:?} takes no colour", theme.name);
        }
    }
}

struct Host {
    focus: gpui_kit::FocusHandle,
}

impl gpui_kit::Render for Host {
    fn render(&mut self, _: &mut gpui_kit::Window, _: &mut gpui_kit::Context<Self>) -> impl gpui_kit::IntoElement {
        use gpui_kit::{ParentElement, Styled};
        gpui_kit::div().p(gpui_kit::px(20.)).child(Button::new("focus-me").label("Focus me").focus_handle(self.focus.clone()).debug_name("focus-me"))
    }
}

#[gpui_kit::test]
fn a_button_focused_from_the_keyboard_draws_the_shared_ring_and_one_focused_by_the_pointer_does_not(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::set_appearance(crate::theme::Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let (host, cx) = cx.add_window_view(|_, cx| Host { focus: cx.focus_handle() });
    cx.simulate_resize(gpui_kit::size(gpui_kit::px(300.), gpui_kit::px(200.)));
    cx.run_until_parked();
    assert!(cx.debug_bounds("button-ring").is_none(), "no focus, no ring");
    let focus = host.read_with(cx, |h, _| h.focus.clone());
    cx.update(|window, cx| focus.focus(window, cx));
    cx.simulate_keystrokes("tab");
    host.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    // The tab moved on from the only stop, so put focus back the way a Tab walk would leave it.
    cx.update(|window, cx| focus.focus(window, cx));
    host.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    assert!(cx.debug_bounds("button-ring").is_some(), "focus from the keyboard shows the ring");
    let at = cx.debug_bounds("focus-me").expect("drawn").center();
    cx.simulate_click(at, gpui_kit::Modifiers::default());
    host.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    assert!(cx.debug_bounds("button-ring").is_none(), "a click is the pointer: no ring");
}

/// A button is a stop in the Tab order without asking: a reader on the keyboard can reach every one.
#[gpui_kit::test]
fn a_button_is_a_tab_stop_by_default(cx: &mut gpui_kit::TestAppContext) {
    use gpui_kit::{IntoElement, ParentElement, Render, Window, div};
    struct Page;
    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
            div().child(Button::new("a").label("First")).child(Button::new("b").label("Second").disabled(true)).child(Button::new("c").label("Third"))
        }
    }
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::set_appearance(crate::theme::Appearance::Light, cx);
    });
    let (_, cx) = cx.add_window_view(|_, _| Page);
    cx.run_until_parked();
    let mut reached = Vec::new();
    for _ in 0..3 {
        cx.update(|window, cx| window.focus_next(cx));
        cx.run_until_parked();
        reached.push(cx.update(|window, cx| window.focused(cx)));
    }
    assert!(reached[0].is_some() && reached[1].is_some(), "two live buttons are two stops");
    assert_ne!(reached[0], reached[1], "and they are different ones");
    assert_eq!(reached[2], reached[0], "the disabled one is skipped, and the walk goes round");
}

/// K3: a trigger whose menu is open shows no tooltip, so the tooltip never covers the menu it belongs to.
#[gpui_kit::test]
fn a_trigger_shows_its_tooltip_on_hover_and_none_while_its_menu_is_open(cx: &mut gpui_kit::TestAppContext) {
    use gpui_kit::{IntoElement, ParentElement, Render, Styled, Window, div, px};
    struct Trigger {
        open: bool,
    }
    impl Render for Trigger {
        fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
            div().p(px(40.)).child(Button::new("more").debug_name("more").label("More").tooltip("More").open(self.open))
        }
    }
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::set_appearance(crate::theme::Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    for open in [false, true] {
        let (_trigger, cx) = cx.add_window_view(move |_, _| Trigger { open });
        cx.simulate_resize(gpui_kit::size(px(300.), px(200.)));
        cx.run_until_parked();
        let at = cx.debug_bounds("more").expect("drawn").center();
        cx.simulate_mouse_move(at, None, gpui_kit::Modifiers::default());
        cx.executor().advance_clock(std::time::Duration::from_secs(2));
        cx.run_until_parked();
        assert_eq!(cx.debug_bounds("tooltip").is_some(), !open, "open {open}: a tooltip only while the menu is shut");
    }
}

/// A button whose menu or picker is open keeps the hover level, and eases back once it shuts.
#[test]
fn an_open_button_holds_the_hover_level() {
    assert_eq!(hover_target(false, false), 0.);
    assert_eq!(hover_target(true, false), 1.);
    assert_eq!(hover_target(false, true), 1.);
    assert_eq!(hover_target(true, true), 1.);
}
