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
