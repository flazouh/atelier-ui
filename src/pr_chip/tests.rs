use super::*;
use crate::pr::PrState;

fn pr(number: u64) -> PrChipData {
    PrChipData { number, repo: "o/r".into(), title: "t".into(), state: PrState::Open, url: "u".into() }
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
