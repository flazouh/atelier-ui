use super::*;
use crate::pr::PrState;

fn pr(number: u64) -> PrChipData {
    PrChipData { number, repo: "o/r".into(), title: "t".into(), state: PrState::Open, url: "u".into(), facts: None }
}

fn known(n: u64) -> Option<PrChipData> {
    (n == 3344).then(|| pr(n))
}

#[test]
fn a_known_number_becomes_a_chip_link_and_an_unknown_one_stays_text() {
    let (text, chips) = link_prs("Fixed in #3344, not #9999.", known);
    assert_eq!(text, "Fixed in [#3344](atelier-pr:3344), not #9999.");
    assert_eq!(chips.keys().copied().collect::<Vec<_>>(), [3344]);
}

#[test]
fn code_and_urls_stay_untouched() {
    let text = "`#3344` and https://x.dev/#3344";
    assert_eq!(link_prs(text, known).0, text);
}

#[test]
fn every_mention_of_a_number_links() {
    let (text, chips) = link_prs("#3344 then #3344", known);
    assert_eq!(text, "[#3344](atelier-pr:3344) then [#3344](atelier-pr:3344)");
    assert_eq!(chips.len(), 1);
}

#[test]
fn a_chip_link_reads_back_as_its_number() {
    assert_eq!(chip_number("atelier-pr:3344"), Some(3344));
    assert_eq!(chip_number("https://example.com"), None);
    assert_eq!(chip_number("atelier-pr:x"), None);
}

#[test]
fn the_size_bar_splits_its_squares_as_github_does() {
    use super::helpers::added_squares;
    assert_eq!(added_squares(0, 0), 0);
    assert_eq!(added_squares(10, 0), 5);
    assert_eq!(added_squares(0, 10), 0);
    assert_eq!(added_squares(50, 50), 3);
    assert_eq!(added_squares(1000, 1), 4, "a side with any lines keeps a square");
    assert_eq!(added_squares(1, 1000), 1, "a side with any lines keeps a square");
}

#[test]
fn a_card_opens_at_once_while_one_is_open_or_just_closed() {
    use std::time::{Duration, Instant};
    use super::{helpers::warm, structs::Warmth};
    let now = Instant::now();
    assert!(!warm(None, now), "the first card waits");
    assert!(warm(Some(&Warmth { open: true, changed: Some(now - Duration::from_secs(30)) }), now));
    assert!(warm(Some(&Warmth { open: false, changed: Some(now - Duration::from_millis(200)) }), now));
    assert!(!warm(Some(&Warmth { open: false, changed: Some(now - Duration::from_secs(2)) }), now));
}
