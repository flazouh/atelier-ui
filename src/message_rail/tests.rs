use super::*;

/// The tick for the message in view is whole; its neighbours are 68%, 44% and then 25% of it.
#[test]
fn ticks_swell_like_a_dock_round_the_lit_one() {
    let scales: Vec<f32> = (0..6).map(|i| tick_scale(i, Some(2))).collect();
    assert_eq!(scales, [0.44, 0.68, 1., 0.68, 0.44, 0.25]);
    assert_eq!(tick_scale(0, None), 0.25, "with none lit every tick is short");
}

/// The label and the answer's start are cut at a word, with an ellipsis, as the web does.
#[test]
fn an_excerpt_is_cut_at_a_word() {
    assert_eq!(excerpt("short text", 56), "short text");
    let long = "Run these shell commands one at a time, each as its own tool call please";
    let cut = excerpt(long, 40);
    assert!(cut.ends_with('…') && cut.chars().count() <= 41, "{cut}");
    assert!(!cut.trim_end_matches('…').ends_with(' '));
    assert_eq!(excerpt("  many   spaces \n here ", 56), "many spaces here");
}

#[test]
fn the_ticks_shrink_when_many_messages_would_not_fit() {
    assert_eq!(item_size(5, 400.), 14.);
    assert_eq!(item_size(100, 400.), 4.);
    assert!((item_size(40, 400.) - 10.).abs() < 1e-4);
}
