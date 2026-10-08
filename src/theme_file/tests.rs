use super::*;

#[test]
fn a_colour_reads_in_each_hex_form() {
    let red = |c: Hsla| c.to_rgb();
    assert_eq!(red(hex("#FF0000").unwrap()).r, 1.);
    assert!((red(hex("#F00").unwrap()).r - 1.).abs() < 1e-6);
    assert!((hex("#00000080").unwrap().a - 128. / 255.).abs() < 1e-6);
    assert!((hex("#0008").unwrap().a - 136. / 255.).abs() < 1e-6);
    assert!(hex("#12345").is_none() && hex("123456").is_none() && hex("#GG0000").is_none());
}

#[test]
fn atelier_light_is_todays_colours() {
    let (theme, derived) = parse(include_str!("../../assets/themes/atelier-light.json")).unwrap();
    assert_eq!(theme.background, hex("#F0EEE6").unwrap());
    assert_eq!(theme.foreground, hex("#141413").unwrap());
    assert_eq!((theme.primary, theme.primary_foreground), (theme.foreground, theme.background), "the page inverted");
    assert_eq!(theme.accent, hex("#F9A825").unwrap());
    assert_eq!(derived, ["selection", "popover", "shadow", "diff_added", "diff_removed"]);
    assert_eq!(theme.chip_rest, hex("#121212").unwrap(), "mem0's chip, as measured");
    assert_eq!(theme.selection, theme.foreground.opacity(0.45), "as before: the ink at 45%");
}
#[test]
fn the_atelier_themes_carry_their_chart_palette() {
    for json in [include_str!("../../assets/themes/atelier-dark.json"), include_str!("../../assets/themes/atelier-light.json")] {
        let (theme, derived) = parse(json).unwrap();
        assert!(!derived.contains(&"chart"), "the palette is in the file");
        assert_ne!(theme.series(0, 0), theme.series(0, 1), "a hue has its shades");
        assert_ne!(theme.series(0, 0), theme.series(1, 0), "the hues differ");
    }
    let (dark, _) = parse(include_str!("../../assets/themes/atelier-dark.json")).unwrap();
    assert_eq!(dark.series(1, 0), hex("#8fb7b3").unwrap(), "Codex, from the mockup");
    assert_eq!(dark.series(0, 2), hex("#f2b79c").unwrap());
    assert_eq!(dark.series(4, 3), dark.series(0, 0), "a larger index wraps round");
}

#[test]
fn a_theme_without_a_chart_derives_it_from_its_status_tones() {
    let (theme, _) = parse(include_str!("../../assets/themes/atelier-dark.json")).unwrap();
    let mut file: serde_json::Value = serde_json::from_str(include_str!("../../assets/themes/atelier-dark.json")).unwrap();
    file.as_object_mut().unwrap().remove("chart");
    let (plain, derived) = parse(&file.to_string()).unwrap();
    assert!(derived.contains(&"chart"));
    assert_eq!(plain.series(0, 0), theme.warning);
    assert_eq!(plain.series(3, 0), theme.danger);
    assert!(plain.series(1, 1).l < plain.series(1, 0).l && plain.series(1, 2).l > plain.series(1, 0).l);
}
