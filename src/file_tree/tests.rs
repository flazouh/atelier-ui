use std::collections::HashSet;

use super::*;
use crate::changed_files::ChangedFile;

fn files() -> Vec<ChangedFile> {
    vec![
        ChangedFile::new("crates/beui/src/theme.rs", 2, 1),
        ChangedFile::new("crates/beui/src/file_diff.rs", 18, 6),
        ChangedFile::new("crates/beui/src/file_diff/tests.rs", 42, 0),
        ChangedFile::new("crates/gallery/src/main.rs", 12, 4),
        ChangedFile::new("README.md", 2, 0),
    ]
}

/// Each visible row as `depth name +a -r`, with a `/` after a folder.
fn lines(rows: &[TreeRow]) -> Vec<String> {
    rows.iter()
        .map(|r| {
            let slash = if r.is_folder() { "/" } else { "" };
            format!(
                "{}{}{slash} +{} -{}",
                "  ".repeat(r.depth),
                r.name,
                r.added,
                r.removed
            )
        })
        .collect()
}

#[test]
fn folders_come_first_then_files_each_by_name_and_single_child_folders_merge() {
    let tree = FileTree::new(&files());
    assert_eq!(
        lines(&tree.rows(&HashSet::new())),
        [
            "crates/ +74 -11",
            "  beui/src/ +62 -7",
            "    file_diff/ +42 -0",
            "      tests.rs +42 -0",
            "    file_diff.rs +18 -6",
            "    theme.rs +2 -1",
            "  gallery/src/ +12 -4",
            "    main.rs +12 -4",
            "README.md +2 -0",
        ]
    );
}

#[test]
fn a_single_file_under_single_folders_compacts_to_one_folder_row() {
    let tree = FileTree::new(&[ChangedFile::new("a/b/c/d.rs", 1, 1)]);
    assert_eq!(
        lines(&tree.rows(&HashSet::new())),
        ["a/b/c/ +1 -1", "  d.rs +1 -1"]
    );
}

#[test]
fn a_folded_folder_hides_what_is_under_it_but_keeps_its_sums() {
    let tree = FileTree::new(&files());
    let folded = HashSet::from([SharedString::from("crates/beui/src")]);
    assert_eq!(
        lines(&tree.rows(&folded)),
        [
            "crates/ +74 -11",
            "  beui/src/ +62 -7",
            "  gallery/src/ +12 -4",
            "    main.rs +12 -4",
            "README.md +2 -0"
        ]
    );
    assert!(tree.rows(&folded)[1].folded);
}

#[test]
fn each_row_knows_its_full_path() {
    let tree = FileTree::new(&files());
    let rows = tree.rows(&HashSet::new());
    assert_eq!(rows[1].path, "crates/beui/src");
    assert_eq!(rows[3].path, "crates/beui/src/file_diff/tests.rs");
}

#[test]
fn the_file_order_is_the_tree_order() {
    let tree = FileTree::new(&files());
    assert_eq!(
        tree.file_order(),
        [
            "crates/beui/src/file_diff/tests.rs",
            "crates/beui/src/file_diff.rs",
            "crates/beui/src/theme.rs",
            "crates/gallery/src/main.rs",
            "README.md",
        ]
    );
}

#[test]
fn an_empty_list_is_an_empty_tree() {
    assert!(FileTree::new(&[]).rows(&HashSet::new()).is_empty());
}

#[test]
fn names_sort_without_regard_to_case() {
    let tree = FileTree::new(&[
        ChangedFile::new("Cargo.toml", 1, 0),
        ChangedFile::new("build.rs", 1, 0),
        ChangedFile::new("apps/x.rs", 1, 0),
        ChangedFile::new("Docs/y.md", 1, 0),
    ]);
    assert_eq!(
        tree.file_order(),
        ["apps/x.rs", "Docs/y.md", "build.rs", "Cargo.toml"]
    );
}
