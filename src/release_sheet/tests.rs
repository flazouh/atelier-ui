use std::{cell::Cell, rc::Rc};

use gpui_kit::{IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, div, px};

use super::{ReleaseNote, ReleaseSheet, ReleaseVersion};

struct Page {
    closed: Rc<Cell<u32>>,
}

impl Render for Page {
    fn render(
        &mut self,
        _: &mut gpui_kit::Window,
        _: &mut gpui_kit::Context<Self>,
    ) -> impl IntoElement {
        let closed = self.closed.clone();
        let earlier = [
            ReleaseVersion::new(
                "0.1.7",
                [ReleaseNote::new("Tall panels", "A panel scrolls.")],
            )
            .date(Some("Oct 8, 2026".into())),
            ReleaseVersion::new("0.1.6", [ReleaseNote::new("Accounts", "Each has a tile.")]),
        ];
        div().w(px(860.)).child(
            ReleaseSheet::new("sheet", "0.1.8")
                .date(Some("Oct 9, 2026".into()))
                .notes([
                    ReleaseNote::new("The stop", "A little red."),
                    ReleaseNote::new("The header", "No Stop."),
                ])
                .earlier(earlier)
                .on_close(move |_, _| closed.set(closed.get() + 1)),
        )
    }
}

fn open(cx: &mut TestAppContext) -> (Rc<Cell<u32>>, &mut gpui_kit::VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
    });
    let closed = Rc::new(Cell::new(0));
    let (_, cx) = cx.add_window_view({
        let closed = closed.clone();
        move |_, _| Page { closed }
    });
    cx.run_until_parked();
    (closed, cx)
}

#[gpui_kit::test]
fn a_note_draws_its_lead_and_its_text_and_nothing_else(cx: &mut TestAppContext) {
    let (_, cx) = open(cx);
    for name in [
        "release-lead-0-0",
        "release-text-0-0",
        "release-lead-0-1",
        "release-text-0-1",
        "release-lead-1-0",
    ] {
        assert!(cx.debug_bounds(name).is_some(), "{name} is drawn");
    }
    let lead = cx.debug_bounds("release-lead-0-0").unwrap();
    let text = cx.debug_bounds("release-text-0-0").unwrap();
    assert!(
        lead.bottom() <= text.top(),
        "the text stands under the lead"
    );
    for name in [
        "release-kind-0",
        "release-icon-0-0",
        "release-footer",
        "release-later",
        "release-install",
    ] {
        assert!(cx.debug_bounds(name).is_none(), "{name} is gone");
    }
}

#[gpui_kit::test]
fn a_release_with_a_date_draws_it_and_one_without_does_not(cx: &mut TestAppContext) {
    let (_, cx) = open(cx);
    assert!(
        cx.debug_bounds("release-date-0").is_some(),
        "the current release has a date"
    );
    assert!(
        cx.debug_bounds("release-date-1").is_some(),
        "0.1.7 has a date"
    );
    assert!(
        cx.debug_bounds("release-date-2").is_none(),
        "0.1.6 has none, and no line for it"
    );
}

#[gpui_kit::test]
fn earlier_releases_stand_in_order_below_the_current_one(cx: &mut TestAppContext) {
    let (_, cx) = open(cx);
    let tops: Vec<_> = ["release-0", "release-1", "release-2"]
        .map(|n| cx.debug_bounds(n).expect("the release is drawn").top())
        .into();
    assert!(
        tops[0] < tops[1] && tops[1] < tops[2],
        "current, then 0.1.7, then 0.1.6: {tops:?}"
    );
    let band = cx.debug_bounds("release-band").expect("the band is drawn");
    assert!(
        band.bottom() <= cx.debug_bounds("release-0").unwrap().top(),
        "the band is over the first release"
    );
    assert_eq!(f32::from(band.size.height), 132., "the band is 132 tall");
}

#[gpui_kit::test]
fn the_title_stands_bottom_left_in_the_band_and_the_close_button_top_right(
    cx: &mut TestAppContext,
) {
    let (_, cx) = open(cx);
    let band = cx.debug_bounds("release-band").unwrap();
    let title = cx
        .debug_bounds("release-title")
        .expect("the title is drawn");
    let close = cx
        .debug_bounds("release-close")
        .expect("the close button is drawn");
    assert!(
        title.left() - band.left() < px(60.) && band.bottom() - title.bottom() < px(40.),
        "bottom left"
    );
    assert!(
        band.right() - close.right() < px(30.) && close.top() - band.top() < px(30.),
        "top right"
    );
}

#[gpui_kit::test]
fn the_close_button_fires_on_close(cx: &mut TestAppContext) {
    let (closed, cx) = open(cx);
    let close = cx
        .debug_bounds("release-close")
        .expect("the close button is drawn")
        .center();
    cx.simulate_click(close, Modifiers::default());
    assert_eq!(closed.get(), 1);
}

#[gpui_kit::test]
fn a_tall_list_makes_a_tall_sheet_for_the_modal_to_scroll(cx: &mut TestAppContext) {
    let (_, cx) = open(cx);
    let sheet = cx.debug_bounds("release-sheet").unwrap();
    let last = cx.debug_bounds("release-2").unwrap();
    assert!(
        last.bottom() <= sheet.bottom(),
        "every release is inside the sheet"
    );
}
