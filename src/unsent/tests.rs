use super::*;

#[test]
fn the_count_never_says_pending_or_draft() {
    assert_eq!(count_text(1), "1 unsent comment");
    assert_eq!(count_text(3), "3 unsent comments");
    assert_eq!(send_text(1), "Send it");
    assert_eq!(send_text(3), "Send all 3");
    for n in 1..4 {
        for words in [count_text(n), send_text(n), SharedString::from(WHO_SEES)] {
            let lower = words.to_lowercase();
            assert!(
                !lower.contains("pending") && !lower.contains("draft"),
                "{words}"
            );
        }
    }
}
