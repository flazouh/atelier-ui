use super::*;
use crate::pr::{PrChipData, PrState};

fn pr(number: u64, court: Court, changed_at: u64) -> CourtItem {
    CourtItem {
        pr: PrChipData { number, repo: "o/r".into(), title: format!("PR {number}").into(), state: PrState::Open, url: "u".into(), facts: None },
        author: "Maya".into(),
        court,
        why: "".into(),
        checks: Default::default(),
        review: Default::default(),
        comments: 0,
        added: 0,
        removed: 0,
        age: "".into(),
        changed_at,
        unread: false,
    }
}

fn shape(items: Vec<CourtItem>) -> Vec<(Court, Vec<u64>)> {
    courts(items).into_iter().map(|(c, rows)| (c, rows.iter().map(|r| r.pr.number).collect())).collect()
}

#[test]
fn courts_come_in_reading_order_and_empty_ones_are_left_out() {
    let items = vec![pr(1, Court::Settled, 1), pr(2, Court::Waiting, 1), pr(3, Court::NeedsYou, 1)];
    assert_eq!(shape(items), [(Court::NeedsYou, vec![3]), (Court::Waiting, vec![2]), (Court::Settled, vec![1])]);
}

#[test]
fn inside_a_court_the_newest_change_comes_first() {
    let items = vec![pr(1, Court::NeedsYou, 10), pr(2, Court::NeedsYou, 30), pr(3, Court::NeedsYou, 20)];
    assert_eq!(shape(items), [(Court::NeedsYou, vec![2, 3, 1])]);
}

#[test]
fn a_pull_request_listed_twice_sits_once_in_its_most_urgent_court() {
    let items = vec![pr(7, Court::Waiting, 5), pr(7, Court::NeedsYou, 5), pr(8, Court::Running, 1)];
    assert_eq!(shape(items), [(Court::NeedsYou, vec![7]), (Court::Running, vec![8])]);
}

#[test]
fn each_court_has_gitquiets_name_and_meaning() {
    assert_eq!(Court::ALL.map(Court::name), ["Needs You", "Waiting", "Running", "Settled"]);
    assert_eq!(Court::Running.means(), "A machine is still working. Nothing to do but wait.");
}

#[test]
fn a_row_counts_its_checks_as_passed_of_all() {
    let c = |passed, failed, running| Checks { passed, failed, running };
    assert_eq!(checks_text(c(11, 0, 7)), Some("11 of 18".into()));
    assert_eq!(checks_text(c(37, 4, 0)), Some("37 of 41".into()));
    assert_eq!(checks_text(c(5, 0, 0)), None, "all passed shows only the mark");
    assert_eq!(checks_text(c(0, 0, 0)), None);
}

/// In a narrow pane the title was pushed to nothing by seven fixed columns, leaving a row of "#8" and no words.
#[gpui_kit::test]
fn in_a_narrow_pane_the_title_keeps_its_room_and_the_other_columns_give_way(cx: &mut gpui_kit::TestAppContext) {
    use gpui_kit::{IntoElement, ParentElement, Render, Styled, Window, div, px, size};
    struct Page(Vec<CourtItem>);
    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
            div().size_full().child(CourtList::new("court", self.0.clone()))
        }
    }
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::set_appearance(crate::theme::Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let mut item = pr(8, Court::NeedsYou, 1);
    item.pr.title = "Add a small crate for the pull request view pass".into();
    item.why = "Ready to merge".into();
    item.pr.repo = "acme/widgets".into();
    let (page, cx) = cx.add_window_view(move |_, _| Page(vec![item]));
    cx.simulate_resize(size(px(650.), px(400.)));
    for _ in 0..3 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    let title = cx.debug_bounds("court-title").expect("the title is drawn");
    assert!(f32::from(title.size.width) >= TITLE_LEAST - 0.5, "the title has {:?}", title.size.width);
}
