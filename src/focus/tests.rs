use gpui_kit::{Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px, size};

use super::*;
use crate::theme::{Theme, set_appearance, Appearance};

#[test]
fn the_ring_reaches_three_to_one_on_the_surfaces_a_field_sits_on_in_every_theme() {
    for theme in crate::themes::all() {
        for surface in [theme.background, theme.card, theme.card_strong] {
            let ring = ring_color(theme, surface);
            assert!(contrast(ring, surface) >= MARK_CONTRAST, "{}: {:.2}", theme.name, contrast(ring, surface));
            // Quiet: no stronger than it must be, so it is not the ink itself where a mix does.
            if contrast(theme.foreground, surface) > 6. {
                assert_ne!(ring, theme.foreground, "{}: the ring is a mix, not the ink", theme.name);
            }
        }
    }
    assert_eq!(RING_WIDTH, 2.);
}

#[test]
fn the_ring_shadow_has_no_blur_and_is_two_pixels_out() {
    let theme = Theme::light();
    let shadow = ring_shadow(&theme, theme.card_strong);
    assert_eq!(shadow.len(), 1);
    assert_eq!((shadow[0].blur_radius, shadow[0].spread_radius, shadow[0].inset), (px(0.), px(2.), false));
}

struct Page {
    one: FocusHandle,
    two: FocusHandle,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().p(px(20.)).flex().flex_col().gap(px(20.)).children([
            Field::new(self.one.clone(), div().id("one").debug_selector(|| "one".into()).track_focus(&self.one).w(px(100.)).h(px(30.))).into_any_element(),
            Field::new(self.two.clone(), div().id("two").track_focus(&self.two).w(px(100.)).h(px(30.))).into_any_element(),
        ])
    }
}

fn open(cx: &mut TestAppContext) -> (Entity<Page>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
    });
    let (page, cx) = cx.add_window_view(|_, cx| Page { one: cx.focus_handle(), two: cx.focus_handle() });
    cx.simulate_resize(size(px(300.), px(300.)));
    cx.run_until_parked();
    (page, cx)
}

#[gpui_kit::test]
fn a_field_draws_the_ring_only_while_focus_is_inside_it(cx: &mut TestAppContext) {
    let (page, cx) = open(cx);
    assert!(cx.debug_bounds("field-ring").is_none(), "no focus, no ring");
    let (one, two) = page.read_with(cx, |p, _| (p.one.clone(), p.two.clone()));
    cx.update(|window, cx| one.focus(window, cx));
    page.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    let first = cx.debug_bounds("field-ring").expect("the ring shows on the focused field");
    cx.update(|window, cx| two.focus(window, cx));
    page.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    let second = cx.debug_bounds("field-ring").expect("and moves with focus");
    assert!(second.top() > first.bottom(), "the ring is on the second field now: {first:?} {second:?}");
}

#[test]
fn the_row_ring_is_the_same_colour_as_the_field_ring() {
    for theme in crate::themes::all() {
        assert_eq!(ring_color(theme, theme.background), ring_shadow(theme, theme.background)[0].color, "{}", theme.name);
    }
}
