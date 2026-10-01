use super::*;

#[test]
fn a_monogram_is_the_labels_first_letter() {
    assert_eq!(monogram_letter("opus 5.5"), "O");
    assert_eq!(monogram_letter("émile"), "É");
    assert_eq!(monogram_letter(""), "?");
}

#[test]
fn a_mark_picks_its_asset_by_theme() {
    let mark = BrandMark::new("labs/a-light.svg", "labs/a-dark.svg");
    assert_eq!(mark.for_theme(Appearance::Light), "labs/a-light.svg");
    assert_eq!(mark.for_theme(Appearance::Dark), "labs/a-dark.svg");
}
