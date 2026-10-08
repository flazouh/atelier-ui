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
    assert!(cx.debug_bounds("release-close").is_some(), "Close is a button of its own");
    assert!(cx.debug_bounds("release-later").is_none() && cx.debug_bounds("release-install").is_none(), "and the foot is gone");
    let sheet = cx.debug_bounds("release-sheet").unwrap();
    let close = cx.debug_bounds("release-close").unwrap();
    assert!(close.right() <= sheet.right() && sheet.right() - close.right() < gpui_kit::px(30.) && close.top() - sheet.top() < gpui_kit::px(30.), "at the top right: {close:?} in {sheet:?}");
}

#[gpui_kit::test]
fn earlier_versions_are_listed_under_the_notes_and_the_list_has_a_height_of_its_own(cx: &mut gpui_kit::TestAppContext) {
    struct Page;
    impl gpui_kit::Render for Page {
        fn render(&mut self, _: &mut gpui_kit::Window, _: &mut gpui_kit::Context<Self>) -> impl gpui_kit::IntoElement {
            use gpui_kit::{ParentElement, Styled, div, px};
            let notes = |n: usize| (0..n).map(|i| super::ReleaseNote::new(format!("Lead {i}."), "Text.")).collect::<Vec<_>>();
            let earlier = (0..6).map(|i| super::ReleaseVersion::new(format!("0.0.{i}"), notes(3)));
            div().w(px(520.)).child(super::ReleaseSheet::new("sheet", "0.1.4").notes(notes(2)).earlier(earlier).labels("Close", "").on_later(|_, _| {}))
        }
    }
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
    });
    let (_page, cx) = cx.add_window_view(|_, _| Page);
    cx.run_until_parked();
    assert!(cx.debug_bounds("release-earlier-0").is_some() && cx.debug_bounds("release-earlier-5").is_some(), "every earlier version is there");
    let sheet = cx.debug_bounds("release-sheet").expect("the sheet is drawn");
    assert!(f32::from(sheet.size.height) < 232. + 340. + 140., "the notes scroll, so the sheet stays short: {:?}", sheet.size);
}

#[test]
fn each_kind_has_its_own_icon_and_its_own_label() {
    let kinds = super::ReleaseKind::ALL;
    for (i, a) in kinds.iter().enumerate() {
        for b in &kinds[i + 1..] {
            assert_ne!(a.icon().name(), b.icon().name(), "{a:?} and {b:?} have two icons");
            assert_ne!(a.label(), b.label(), "{a:?} and {b:?} have two labels");
        }
    }
    assert_eq!(kinds.len(), 6);
}

struct Kinds {
    asked: std::rc::Rc<std::cell::RefCell<Vec<super::ReleaseKind>>>,
}
impl gpui_kit::Render for Kinds {
    fn render(&mut self, _: &mut gpui_kit::Window, _: &mut gpui_kit::Context<Self>) -> impl gpui_kit::IntoElement {
        use gpui_kit::{ParentElement, Styled, div, px};
        let asked = self.asked.clone();
        let mut sheet = super::ReleaseSheet::new("sheet", "0.1.4").labels("Close", "Restart").on_later(|_, _| {});
        for kind in super::ReleaseKind::ALL {
            sheet = sheet.note(super::ReleaseNote::new(kind.label(), "What it says.").kind(kind));
        }
        sheet = sheet.note(super::ReleaseNote::new("No kind", "A note with none."));
        let sheet = sheet.kind_colors(move |kind, _| {
            asked.borrow_mut().push(kind);
            gpui_kit::hsla(0.5, 0.8, 0.6, 1.)
        });
        div().w(px(520.)).child(sheet)
    }
}
/// A note with a kind has its label over the lead, in the colour the owner gave it; a note with none has no label.
#[gpui_kit::test]
fn a_note_with_a_kind_has_a_label_and_the_owner_gives_the_colour(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
    });
    let asked: std::rc::Rc<std::cell::RefCell<Vec<super::ReleaseKind>>> = Default::default();
    let (_page, cx) = cx.add_window_view({
        let asked = asked.clone();
        move |_, _| Kinds { asked }
    });
    cx.run_until_parked();
    for (i, name) in ["release-kind-0", "release-kind-1", "release-kind-2", "release-kind-3", "release-kind-4", "release-kind-5"].into_iter().enumerate() {
        assert!(cx.debug_bounds(name).is_some(), "note {i} has its kind label");
    }
    assert!(cx.debug_bounds("release-kind-6").is_none(), "a note with no kind has none");
    let asked = asked.borrow();
    for kind in super::ReleaseKind::ALL {
        assert!(asked.contains(&kind), "the owner was asked for the colour of {kind:?}");
    }
}
