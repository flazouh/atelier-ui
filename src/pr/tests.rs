use super::*;

fn checks(passed: u32, failed: u32, running: u32) -> ChecksSummary {
    Checks { passed, failed, running }.summary()
}

#[test]
fn the_checks_summary_names_the_worst_news_first() {
    let cases = [
        ((3, 0, 0), "3 checks passed"),
        ((1, 0, 0), "1 check passed"),
        ((3, 1, 0), "1 failing"),
        ((0, 2, 4), "2 failing"),
        ((3, 0, 2), "Checks running"),
        ((0, 0, 0), ""),
    ];
    for ((passed, failed, running), text) in cases {
        assert_eq!(checks(passed, failed, running).text(), text, "{passed} {failed} {running}");
    }
}

#[test]
fn review_states_read_as_short_words() {
    assert_eq!(ReviewState::Approved.text(), "Approved");
    assert_eq!(ReviewState::ChangesRequested.text(), "Changes asked");
    assert_eq!(ReviewState::Requested.text(), "Review asked");
    assert_eq!(ReviewState::None.text(), "");
}

#[test]
fn each_state_has_its_word_and_its_own_mark() {
    let states = [PrState::Open, PrState::Draft, PrState::Merged, PrState::Closed];
    assert_eq!(states.map(PrState::label), ["Open", "Draft", "Merged", "Closed"]);
    let mut icons: Vec<_> = states.iter().map(|s| s.icon().name()).collect();
    icons.sort();
    icons.dedup();
    assert_eq!(icons.len(), 4);
}

#[test]
fn a_chip_reads_as_its_number() {
    let pr = PrChipData { number: 3344, repo: "o/r".into(), title: "t".into(), state: PrState::Open, url: "u".into() };
    assert_eq!(pr.label(), "#3344");
}
