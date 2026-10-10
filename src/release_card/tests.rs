use std::{cell::Cell, rc::Rc};

use gpui_kit::{
    AssetSource, Context, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, Window, div, px,
};

use super::{HERO_PATH, ReleaseCard, ReleaseCardNote};
use crate::{
    Assets, IconName,
    theme::{Appearance, set_appearance},
};

#[test]
fn the_assets_serve_the_hero_picture() {
    let bytes = Assets.load(HERO_PATH).expect("the source answers").expect("the picture is served");
    assert!(bytes.starts_with(&[0xFF, 0xD8]), "a jpeg");
}

struct Host {
    secondary: Rc<Cell<u32>>,
    primary: Rc<Cell<u32>>,
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (a, b) = (self.secondary.clone(), self.primary.clone());
        div().w(px(440.)).child(
            ReleaseCard::new("card", "atelier", "0.1.16")
                .title("What\u{2019}s new in atelier")
                .date(Some("Version 0.1.16".into()))
                .notes([ReleaseCardNote::new(IconName::BarChart, "Usage is a full view"), ReleaseCardNote::new(IconName::Dns, "SSH opens again")])
                .on_secondary(move |_, _| a.set(a.get() + 1))
                .on_primary(move |_, _| b.set(b.get() + 1)),
        )
    }
}

#[gpui_kit::test]
fn the_two_buttons_tell_the_app_which_one_was_pressed(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let (secondary, primary) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
    let host = Host { secondary: secondary.clone(), primary: primary.clone() };
    let (_, cx) = cx.add_window_view(move |_, _| host);
    for _ in 0..3 {
        cx.run_until_parked();
    }
    assert!(cx.debug_bounds("release-card-hero").is_some(), "the picture is drawn");
    let all = cx.debug_bounds("release-card-secondary").expect("the quiet button is drawn");
    cx.simulate_click(all.center(), Modifiers::default());
    let got = cx.debug_bounds("release-card-primary").expect("the main button is drawn");
    cx.simulate_click(got.center(), Modifiers::default());
    assert_eq!((secondary.get(), primary.get()), (1, 1));
    assert!(got.left() > all.right(), "the main button is at the right of the quiet one");
}
