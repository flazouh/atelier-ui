use super::*;

#[test]
fn only_image_files_are_offered() {
    assert!(is_icon_file("public/favicon.ico"));
    assert!(is_icon_file("logo.SVG"));
    assert!(is_icon_file("assets/mark.png"));
    assert!(!is_icon_file("src/main.rs"));
    assert!(!is_icon_file("README.md"));
    assert!(!is_icon_file("logo"), "a name with no extension is not an image");
}

/// A name a project's mark tends to have, in a place it tends to live, comes first; depth and name settle the rest.
#[test]
fn a_projects_own_mark_ranks_above_a_vendored_icon_set() {
    let paths = [
        "node_modules/lucide/icons/arrow.svg",
        "docs/screenshots/home.png",
        "assets/logo.svg",
        "favicon.ico",
        "src/deep/er/icon.png",
        "public/hero.png",
    ];
    let ranked = rank(&paths);
    assert_eq!(ranked[0], "favicon.ico", "named and at the root");
    assert_eq!(ranked[1], "assets/logo.svg", "named and in a conventional folder, one level down");
    assert_eq!(ranked[2], "src/deep/er/icon.png", "named, but not in a conventional folder");
    assert_eq!(ranked[3], "public/hero.png", "in a conventional folder, not named");
    assert!(ranked.contains(&"node_modules/lucide/icons/arrow.svg".to_string()));
    assert_eq!(ranked.last().map(String::as_str), Some("node_modules/lucide/icons/arrow.svg"));
}

#[test]
fn the_ranking_is_total_so_a_reload_shows_the_same_list() {
    let a = rank(&["b/logo.png", "a/logo.png", "logo.svg", "logo.png"]);
    let b = rank(&["logo.png", "logo.svg", "a/logo.png", "b/logo.png"]);
    assert_eq!(a, b);
}

#[test]
fn every_typed_word_must_be_in_the_path() {
    let paths = vec!["assets/logo.svg".to_string(), "public/logo.png".to_string(), "public/hero.png".to_string()];
    assert_eq!(filter(&paths, "logo png"), ["public/logo.png"]);
    assert_eq!(filter(&paths, "PUBLIC"), ["public/logo.png", "public/hero.png"]);
    assert_eq!(filter(&paths, "  ").len(), 3, "blank words keep everything");
}
