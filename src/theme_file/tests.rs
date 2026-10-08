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
