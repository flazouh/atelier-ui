use super::helpers::fade;
use crate::theme::Theme;

#[test]
fn a_fade_keeps_the_colour_and_scales_only_the_strength() {
    let colour = Theme::dark().foreground;
    let half = fade(colour, 0.5);
    assert_eq!((half.h, half.s, half.l), (colour.h, colour.s, colour.l));
    assert!((half.a - colour.a * 0.5).abs() < 1e-6);
}

struct Page {
    install: bool,
}
impl gpui_kit::Render for Page {
    fn render(&mut self, _: &mut gpui_kit::Window, _: &mut gpui_kit::Context<Self>) -> impl gpui_kit::IntoElement {
        use gpui_kit::{ParentElement, Styled, div, px};
        let sheet = super::ReleaseSheet::new("sheet", "0.1.4")
            .note(super::ReleaseNote::new("A lead.", "What it says."))
            .labels("Close", "Restart")
            .on_later(|_, _| {});
        div().w(px(520.)).child(if self.install { sheet.on_install(|_, _| {}) } else { sheet })
    }
}

#[gpui_kit::test]
fn with_nothing_to_restart_the_sheet_has_only_its_close_button(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
    });
    let (page, cx) = cx.add_window_view(|_, _| Page { install: true });
    cx.run_until_parked();
    assert!(cx.debug_bounds("release-later").is_some() && cx.debug_bounds("release-install").is_some(), "both buttons with a restart");
    page.update(cx, |p, cx| {
        p.install = false;
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("release-later").is_some(), "Close stays");
    assert!(cx.debug_bounds("release-install").is_none(), "and there is nothing to restart");
}
