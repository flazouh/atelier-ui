use super::helpers::{added_squares, short_path};
use super::*;

#[test]
fn the_size_bar_splits_its_squares_as_github_does() {
    assert_eq!(added_squares(0, 0), 0);
    assert_eq!(added_squares(10, 0), 5);
    assert_eq!(added_squares(0, 10), 0);
    assert_eq!(added_squares(50, 50), 3);
    assert_eq!(added_squares(1000, 1), 4, "a side with any lines keeps a square");
    assert_eq!(added_squares(1, 1000), 1, "a side with any lines keeps a square");
}

#[test]
fn every_part_shows_until_hidden() {
    let parts = PrParts::default();
    assert!(PrPart::ALL.iter().all(|p| parts.shows(*p)));
    assert!(parts.hidden().is_empty());
}

#[test]
fn hidden_parts_round_trip_through_their_keys() {
    let mut parts = PrParts::default();
    parts.set(PrPart::Files, false);
    parts.set(PrPart::Live, false);
    let hidden = parts.hidden();
    assert_eq!(hidden, ["files", "live"]);
    assert_eq!(PrParts::without(&hidden), parts);
    parts.set(PrPart::Files, true);
    assert!(parts.shows(PrPart::Files));
}

#[test]
fn an_unknown_key_in_settings_hides_nothing() {
    assert_eq!(PrParts::without(&["gone".into()]), PrParts::default());
}

#[test]
fn every_part_has_its_own_key() {
    let mut keys: Vec<_> = PrPart::ALL.iter().map(|p| p.key()).collect();
    keys.dedup();
    assert_eq!(keys.len(), PrPart::ALL.len());
    assert!(PrPart::ALL.iter().all(|p| PrPart::from_key(p.key()) == Some(*p)));
}

#[test]
fn the_top_files_are_the_biggest_three() {
    let files = vec![("a", 3), ("b", 40), ("c", 1), ("d", 40), ("e", 9)];
    assert_eq!(top_files(files, |f| f.1), [("b", 40), ("d", 40), ("e", 9)]);
}

#[test]
fn a_long_path_keeps_its_file_name() {
    assert_eq!(short_path("src/a.rs", 20), "src/a.rs");
    assert_eq!(short_path("crates/forge/src/github/briefs/helpers.rs", 20), "…/briefs/helpers.rs");
    assert!(short_path("crates/forge/src/github/briefs/helpers.rs", 20).chars().count() <= 20);
    assert_eq!(short_path("a/very_long_file_name_here.rs", 10), "…e_here.rs");
}

/// What one draw of a card costs with every part showing: layout, prepaint and paint of the card alone.
/// Target: under 1 ms, an eighth of a frame; 0.8 ms on the HP in release.
///     cargo test --release --lib pr_glance::tests::one_draw -- --ignored --nocapture
#[gpui_kit::test]
#[ignore]
fn one_draw_of_a_full_card(cx: &mut gpui_kit::TestAppContext) {
    use gpui_kit::{AvailableSpace, ParentElement, point, px, size};
    use crate::pr::{Checks, PrChipData, PrFacts, PrReviewer, PrStanding, PrState, PrVerdict, ReviewState, StandingTone};
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::set_appearance(crate::theme::Appearance::Dark, cx);
    });
    let pr = PrChipData {
        number: 3311,
        repo: "flazouh/atelier".into(),
        title: "fix(relay): detach the byte stream before a second write".into(),
        state: PrState::Open,
        url: "https://github.com/flazouh/atelier/pull/3311".into(),
        facts: Some(PrFacts {
            author: "flazouh".into(),
            added: 79,
            removed: 10,
            comments: 4,
            review: ReviewState::Requested,
            checks: Some(Checks { passed: 9, failed: 1, running: 0 }),
            updated_at: 1,
            head: "relay-abort".into(),
            base: "main".into(),
            conflicting: false,
            reviewers: vec![PrReviewer { who: "ana".into(), verdict: PrVerdict::Approved }, PrReviewer { who: "core".into(), verdict: PrVerdict::Waiting }],
            standing: Some(PrStanding { tone: StandingTone::Held, word: "Blocked".into(), detail: "a required check fails".into() }),
        }),
    };
    struct Empty;
    impl gpui_kit::Render for Empty {
        fn render(&mut self, _: &mut gpui_kit::Window, _: &mut gpui_kit::Context<Self>) -> impl gpui_kit::IntoElement {
            gpui_kit::div()
        }
    }
    // The card is not the window's root, or every draw would draw it twice.
    let (_, cx) = cx.add_window_view(|_, _| Empty);
    let card = cx.update(|_, cx| gpui_kit::AppContext::new(cx, |cx| PrGlanceCard::new("bench", pr.clone(), None, cx)));
    cx.update(|_, cx| {
        pr_cards(cx).update(cx, |s, cx| {
            s.update_glance(key_of(&pr), |g| {
                g.failing = Some(PrFailing { name: "test (linux-x64)".into(), line: Some("error[E0308]: mismatched types".into()), url: Some("u".into()) });
                g.files = Some((0..3).map(|i| PrFile { path: format!("crates/relay/src/file_{i}.rs").into(), added: 10, removed: 2 }).collect());
                g.session = Some(PrSession { title: "Detach the stream on abort".into(), status: "Idle".into(), running: false });
            }, cx)
        })
    });
    let space = size(AvailableSpace::Definite(px(800.)), AvailableSpace::Definite(px(800.)));
    for _ in 0..20 {
        cx.draw(point(px(0.), px(0.)), space, |_, _| gpui_kit::div().child(card.clone()));
    }
    let mut samples: Vec<std::time::Duration> = (0..200)
        .map(|_| {
            let at = std::time::Instant::now();
            card.update(cx, |_, cx| cx.notify());
            cx.draw(point(px(0.), px(0.)), space, |_, _| gpui_kit::div().child(card.clone()));
            at.elapsed()
        })
        .collect();
    samples.sort();
    let mut empty: Vec<std::time::Duration> = (0..200)
        .map(|_| {
            let at = std::time::Instant::now();
            cx.draw(point(px(0.), px(0.)), space, |_, _| gpui_kit::div());
            at.elapsed()
        })
        .collect();
    empty.sort();
    let us = |d: std::time::Duration| d.as_secs_f64() * 1e6;
    println!("an empty draw: median {:.0} µs", us(empty[100]));
    println!("full card draw: median {:.0} µs, p95 {:.0} µs, target < 1000 µs", us(samples[100]), us(samples[190]));
}
