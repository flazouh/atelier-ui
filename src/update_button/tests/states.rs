use gpui_kit::{IntoElement, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use crate::{
    theme::{Appearance, Theme, set_appearance},
    update_button::{UpdateButton, UpdateState, helpers::look},
};

struct Page(UpdateButton);
impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        div().p(px(20.)).child(self.0.clone())
    }
}

fn shown(button: UpdateButton, cx: &mut TestAppContext) -> &mut VisualTestContext {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(|_, _| Page(button));
    cx.simulate_resize(gpui_kit::size(px(400.), px(200.)));
    cx.run_until_parked();
    cx
}

#[gpui_kit::test]
fn each_state_draws_its_own_selector_and_no_other(cx: &mut TestAppContext) {
    let all = ["update-button-downloading", "update-button-ready", "update-button-restarting"];
    let builds: [(UpdateButton, &str); 3] = [
        (UpdateButton::new("u").downloading(0.55, "Updating 55%"), all[0]),
        (UpdateButton::new("u").ready("Update to v0.1.9"), all[1]),
        (UpdateButton::new("u").restarting("Restarting…"), all[2]),
    ];
    for (button, want) in builds {
        let cx = shown(button, cx);
        assert!(cx.debug_bounds("update-button").is_some(), "{want}: the button is drawn");
        for name in all {
            assert_eq!(cx.debug_bounds(name).is_some(), name == want, "{want}: {name}");
        }
    }
}

#[gpui_kit::test]
fn the_button_is_as_tall_as_the_metric_says(cx: &mut TestAppContext) {
    let cx = shown(UpdateButton::new("u").ready("Update to v0.1.9"), cx);
    let bounds = cx.debug_bounds("update-button").expect("drawn");
    assert_eq!(f32::from(bounds.size.height), 28.);
}

#[test]
fn downloading_is_the_ink_wash_and_ready_is_the_primary_inverted() {
    let theme = Theme::of(Appearance::Dark);
    let wash = look(UpdateState::Downloading(0.5), &theme, 0.);
    assert_eq!(wash.fill, theme.foreground.opacity(0.07));
    assert_eq!(wash.text, theme.foreground);
    assert_eq!(wash.arc, theme.foreground);
    assert_eq!(wash.track, theme.foreground.opacity(0.15));
    assert_eq!(look(UpdateState::Restarting, &theme, 1.).fill, wash.fill, "no hover on a state that cannot be pressed");
    let ready = look(UpdateState::Ready, &theme, 0.);
    assert_eq!((ready.fill, ready.text), (theme.primary, theme.primary_foreground));
    assert_eq!(look(UpdateState::Ready, &theme, 1.).fill, theme.primary_hover());
}

#[test]
fn the_fraction_is_held_to_zero_and_one() {
    assert_eq!(UpdateState::downloading(1.4), UpdateState::Downloading(1.));
    assert_eq!(UpdateState::downloading(-1.), UpdateState::Downloading(0.));
}
