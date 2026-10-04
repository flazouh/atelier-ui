use super::*;

#[test]
fn counts_read_with_a_decimal_only_when_they_have_one() {
    assert_eq!(precise(950), "950");
    assert_eq!(precise(4_100), "4.1K");
    assert_eq!(precise(106_300), "106.3K");
    assert_eq!(precise(300_000), "300K");
    assert_eq!(precise(1_250_000), "1.2M");
}

#[test]
fn the_header_tells_the_share_and_the_numbers() {
    assert_eq!(header(106_300, 300_000), ("35% Full".into(), "~106.3K / 300K Tokens".into()));
    assert_eq!(header(0, 200_000), ("0% Full".into(), "~0 / 200K Tokens".into()));
}

#[test]
fn the_parts_add_up_and_the_window_keeps_the_rest() {
    let parts = [ContextPart::new("System prompt", 4_100), ContextPart::new("Conversation", 69_700)];
    assert_eq!(totals(&parts, 300_000), (73_800, 226_200));
    assert_eq!(totals(&parts, 50_000), (73_800, 0), "past the window leaves nothing, not less than nothing");
}

#[test]
fn a_part_is_its_share_of_the_window() {
    let parts = [ContextPart::new("a", 75_000), ContextPart::new("b", 150_000)];
    assert_eq!(shares(&parts, 300_000), [0.25, 0.5]);
}

#[test]
fn parts_past_the_window_shrink_to_fill_it_exactly() {
    let parts = [ContextPart::new("a", 150_000), ContextPart::new("b", 150_000)];
    assert_eq!(shares(&parts, 100_000), [0.5, 0.5]);
    assert_eq!(shares(&[], 0), Vec::<f32>::new());
}

#[test]
fn the_panel_grows_by_a_row_at_a_time() {
    assert_eq!(height(0), height(1), "an empty list still shows one row");
    assert_eq!(height(3) - height(2), 24.);
}
