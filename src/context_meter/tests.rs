use super::*;

#[test]
fn the_fraction_is_the_share_of_the_window_kept_between_empty_and_full() {
    assert_eq!(fraction(0, 200_000), 0.);
    assert!((fraction(50_000, 200_000) - 0.25).abs() < 1e-6);
    assert_eq!(
        fraction(300_000, 200_000),
        1.,
        "past the window is full, not more"
    );
    assert_eq!(fraction(10, 0), 1., "an empty window is full");
}

#[test]
fn the_ring_turns_amber_at_80_percent_and_red_at_95() {
    assert_eq!(level(0.79), Level::Room);
    assert_eq!(level(WARN_AT), Level::Filling);
    assert_eq!(level(0.94), Level::Filling);
    assert_eq!(level(FULL_AT), Level::Full);
    assert_eq!(level(1.), Level::Full);
}

#[test]
fn token_counts_read_short() {
    assert_eq!(tokens(950), "950");
    assert_eq!(tokens(84_321), "84k");
    assert_eq!(tokens(200_000), "200k");
    assert_eq!(tokens(1_000_000), "1M");
    assert_eq!(tokens(1_250_000), "1.2M");
}

#[test]
fn the_hover_tells_the_numbers_and_the_share() {
    assert_eq!(summary(84_000, 200_000), "84k of 200k tokens (42%)");
    assert_eq!(summary(0, 1_000_000), "0 of 1M tokens (0%)");
}
