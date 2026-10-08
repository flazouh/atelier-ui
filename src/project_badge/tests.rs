use super::*;

#[test]
fn the_palette_is_acepes_twelve_in_its_order() {
    let names: Vec<_> = palette().iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Red", "Orange", "Amber", "Yellow", "Lime", "Green", "Teal", "Cyan", "Blue", "Indigo",
            "Purple", "Pink"
        ]
    );
    assert_eq!(palette().len(), COUNT);
    assert_eq!(palette()[0].rgb, 0xFF5D5A);
    assert_eq!(palette()[11].rgb, 0xFF78F7);
}

/// A project keeps its colour from run to run: the colour follows its place, not chance.
#[test]
fn a_project_colour_follows_its_place_and_stays() {
    let a = fallback_color("/Users/user/code/atelier");
    assert_eq!(a, fallback_color("/Users/user/code/atelier"));
    assert!(a < COUNT);
    // FNV-1a (64 bit) of "a" is 0xaf63dc4c8601ec8c; pinned so a change of hash shows up.
    assert_eq!(fallback_color("a"), 0xaf63dc4c8601ec8c_u64 as usize % COUNT);
    let spread: std::collections::BTreeSet<_> = (0..60)
        .map(|i| fallback_color(&format!("/work/project-{i}")))
        .collect();
    assert!(
        spread.len() >= 8,
        "sixty places use {} of twelve colours",
        spread.len()
    );
}

#[test]
fn a_chosen_colour_beats_the_fallback() {
    assert_eq!(color_of(Some(3), "/x"), 3);
    assert_eq!(color_of(None, "/x"), fallback_color("/x"));
    assert_eq!(
        color_of(Some(99), "/x"),
        fallback_color("/x"),
        "a stranger index falls back"
    );
}

/// One letter, two when names collide on the first, and no more than two.
#[test]
fn a_label_grows_one_letter_when_names_collide() {
    let labels = labels(&[("a", "Acepe"), ("b", "Apple"), ("c", "zed"), ("d", "Acepe")]);
    assert_eq!(
        labels["a"], "Ac",
        "the two Acepe share a label: it stops at two"
    );
    assert_eq!(labels["b"], "Ap");
    assert_eq!(labels["c"], "Z", "a lone name is its first letter, capital");
    assert_eq!(labels["d"], "Ac");
}

#[test]
fn a_label_keeps_the_case_after_its_capital() {
    let both = labels(&[("a", "iPhone"), ("b", "iOS")]);
    assert_eq!(
        (both["a"].as_str(), both["b"].as_str()),
        ("IP", "IO"),
        "the first is a capital and the second stays as the name has it"
    );
    assert_eq!(labels(&[("a", "iPhone")])["a"], "I");
    assert_eq!(
        labels(&[("x", "")])["x"],
        "",
        "a nameless project has no letter"
    );
}

/// Dark ink on the bright palette reads in every colour.
#[test]
fn the_letter_reads_on_every_colour() {
    for (index, swatch) in palette().iter().enumerate() {
        let fill = fill(index);
        let c = crate::theme::contrast(ink_on(fill), fill);
        assert!(c >= 4.5, "{}: {c:.2}", swatch.name);
    }
}
