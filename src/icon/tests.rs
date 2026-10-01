use super::*;

#[test]
fn our_own_icon_is_served_before_gpui_kits() {
    // gpui-kit ships a `close` too; the editor search bar gets ours.
    let ours = Assets.load("icons/close.svg").expect("loads").expect("present");
    assert_eq!(ours.as_ref(), include_bytes!("../../assets/icons/close.svg"));
}

#[test]
fn an_icon_only_gpui_component_draws_comes_from_gpui_kit() {
    // The editor search bar draws these. Without them its buttons are empty squares.
    for path in ["icons/case-sensitive.svg", "icons/replace.svg", "icons/chevron-left.svg"] {
        let bytes = Assets.load(path).expect("loads").unwrap_or_else(|| panic!("{path} is served"));
        assert!(bytes.starts_with(b"<svg"), "{path} is an svg");
    }
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
