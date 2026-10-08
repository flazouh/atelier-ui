use gpui_kit::{
    AppContext as _, Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled,
    TestAppContext, VisualTestContext, Window, component::input::InputState, div, px,
};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn the_shake_goes_left_right_and_settles_over_450_ms() {
    assert!(shake_offset(0.).abs() < 1e-3);
    assert!(
        (shake_offset(SHAKE_SECONDS / 6.) - -6.).abs() < 1e-3,
        "the first stop"
    );
    assert!(
        (shake_offset(SHAKE_SECONDS / 3.) - 6.).abs() < 1e-3,
        "the second"
    );
    assert!(shake_offset(SHAKE_SECONDS * 0.25).abs() <= 6.);
    assert!(
        shake_offset(SHAKE_SECONDS).abs() < 1e-3,
        "back where it started"
    );
    assert!(
        shake_offset(SHAKE_SECONDS * 3.).abs() < 1e-3,
        "past the end stays there"
    );
}

#[test]
fn the_field_is_the_card_strong_step_and_has_no_border_in_every_theme() {
    for theme in crate::themes::all() {
        for surface in [theme.background, theme.card, theme.popover] {
            assert_eq!(
                fill(theme, surface),
                theme.card_strong,
                "{} on {surface:?}",
                theme.name
            );
        }
    }
}
struct Page {
    input: Entity<InputState>,
    error: Option<&'static str>,
    reserve: bool,
    labelled: bool,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let mut field = TextInput::new("f", &self.input)
            .debug_name("field")
            .reserve_error_line(self.reserve);
        if self.labelled {
            field = field.label("Host");
        }
        if let Some(words) = self.error {
            field = field.error(words);
        }
        div()
            .w(px(400.))
            .p(px(20.))
            .child(field)
            .child(div().debug_selector(|| "below".into()).h(px(10.)))
    }
}

fn open<'a>(
    error: Option<&'static str>,
    reserve: bool,
    labelled: bool,
    cx: &'a mut TestAppContext,
) -> (Entity<Page>, &'a mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let (page, cx) = cx.add_window_view(move |window, cx| Page {
        input: cx.new(|cx| InputState::new(window, cx)),
        error,
        reserve,
        labelled,
    });
    for _ in 0..3 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    (page, cx)
}

#[gpui_kit::test]
fn the_field_is_28px_and_the_label_sits_6px_over_it(cx: &mut TestAppContext) {
    let (_, cx) = open(None, false, true, cx);
    let field = cx.debug_bounds("field").unwrap();
    assert_eq!(f32::from(field.size.height), HEIGHT);
    assert_eq!(
        f32::from(field.size.width),
        360.,
        "the page's 400 less its 20px padding"
    );
}

#[gpui_kit::test]
fn a_reserved_error_line_keeps_the_room_so_nothing_below_moves(cx: &mut TestAppContext) {
    let (page, cx) = open(None, true, false, cx);
    let before = cx.debug_bounds("below").unwrap();
    page.update(cx, |p, cx| {
        p.error = Some("Enter a host");
        cx.notify();
    });
    cx.run_until_parked();
    let after = cx.debug_bounds("below").unwrap();
    assert_eq!(
        before.origin.y, after.origin.y,
        "the message came into room that was kept"
    );
    assert_eq!(f32::from(after.origin.y - before.origin.y), 0.);
}

#[gpui_kit::test]
fn without_a_reserved_line_the_message_pushes_what_is_below_down_by_its_line_and_gap(
    cx: &mut TestAppContext,
) {
    let (page, cx) = open(None, false, false, cx);
    let before = cx.debug_bounds("below").unwrap();
    page.update(cx, |p, cx| {
        p.error = Some("Enter a host");
        cx.notify();
    });
    cx.run_until_parked();
    let after = cx.debug_bounds("below").unwrap();
    assert_eq!(
        f32::from(after.origin.y - before.origin.y),
        MESSAGE_LINE + GAP
    );
}

#[test]
fn the_field_keeps_the_old_corner_and_label() {
    assert_eq!((CORNER, GAP, HEIGHT, TEXT_INSET), (8., 4., 28., 10.));
}
