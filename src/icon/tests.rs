use super::*;

#[test]
fn the_icons_gpui_component_draws_for_us_are_ours() {
    // gpui-kit ships a `close` too, from another set; the editor search bar gets ours.
    let ours = Assets.load("icons/close.svg").expect("loads").expect("present");
    assert_eq!(ours.as_ref(), include_bytes!("../../assets/icons/cancel-01.svg"));
    for (path, name) in COMPONENT_ICONS {
        let bytes = Assets.load(path).expect("loads").unwrap_or_else(|| panic!("{path} is served"));
        assert_eq!(bytes.as_ref(), icon_bytes(name.path()).unwrap(), "{path} is ours");
    }
}

#[test]
fn every_icon_is_a_hugeicon() {
    // tools/hugeicons.sh writes each on the 24 grid with Hugeicons' 1.5 stroke; another set's file would not match.
    for name in IconName::ALL {
        let text = std::str::from_utf8(icon_bytes(name.path()).unwrap()).unwrap();
        assert!(text.contains(r#"viewBox="0 0 24 24""#), "{} is on the 24 grid", name.name());
        assert!(text.contains(r#"stroke-width="1.5""#), "{} has Hugeicons' stroke", name.name());
    }
}

#[test]
fn an_icon_we_do_not_have_still_comes_from_gpui_kit() {
    // Nothing should draw as an empty square, even a part atelier has not used before.
    let bytes = Assets.load("icons/calendar.svg").expect("loads").expect("gpui-kit has it");
    assert!(bytes.starts_with(b"<svg"));
}

#[test]
fn the_atelier_mark_is_served() {
    let mark = Assets.load(crate::atelier_mark::PATH).expect("loads").expect("present");
    assert_eq!(mark.as_ref(), include_bytes!("../../assets/atelier-mark.svg"));
}

#[test]
fn an_unknown_path_is_none_not_an_error() {
    assert!(Assets.load("icons/no-such-icon.svg").expect("never an error").is_none());
}
