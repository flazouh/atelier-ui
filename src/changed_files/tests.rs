use super::*;

fn file(path: &str, added: usize, removed: usize) -> ChangedFile {
    ChangedFile::new(path, added, removed)
}

#[test]
fn the_header_counts_files_and_lines() {
    let files = [file("a.rs", 3, 1), file("b.rs", 10, 0), file("c.rs", 0, 4)];
    assert_eq!(header_text(files.len()), "3 files changed");
    assert_eq!(header_text(1), "1 file changed");
    assert_eq!(totals(&files), (13, 5));
}

#[test]
fn a_long_list_folds_after_five_rows() {
    assert_eq!(fold(12, false), Fold { shown: 5, hidden: 7 });
    assert_eq!(fold(7, false), Fold { shown: 5, hidden: 2 });
    assert_eq!(fold(12, true), Fold { shown: 12, hidden: 0 });
}

#[test]
fn a_fold_never_hides_a_single_row() {
    // "Show 1 more" takes the same room as the row it hides.
    assert_eq!(fold(6, false), Fold { shown: 6, hidden: 0 });
    assert_eq!(fold(5, false), Fold { shown: 5, hidden: 0 });
    assert_eq!(fold(0, false), Fold { shown: 0, hidden: 0 });
}

#[test]
fn the_fold_button_says_how_many_rows_it_holds() {
    assert_eq!(fold_label(12, false), Some("Show 7 more".into()));
    assert_eq!(fold_label(12, true), Some("Show fewer".into()));
    assert_eq!(fold_label(6, false), None);
    assert_eq!(fold_label(6, true), None);
}

#[test]
fn a_path_splits_into_its_folder_and_its_name() {
    assert_eq!(split_path("crates/beui/src/theme.rs"), ("crates/beui/src/", "theme.rs"));
    assert_eq!(split_path("Cargo.toml"), ("", "Cargo.toml"));
    assert_eq!(split_path("docs/"), ("docs/", ""));
}

#[test]
fn only_added_deleted_and_renamed_files_carry_a_word() {
    assert_eq!(FileChange::Modified.word(), None);
    assert_eq!(FileChange::Added.word(), Some("Added"));
    assert_eq!(FileChange::Deleted.word(), Some("Deleted"));
    assert_eq!(FileChange::Renamed { from: "old.rs".into() }.word(), Some("Renamed"));
}

#[test]
fn review_starts_at_the_first_file() {
    let files = [file("a.rs", 1, 0), file("b.rs", 1, 0)];
    assert_eq!(first_path(&files).map(|p| p.to_string()), Some("a.rs".into()));
    assert_eq!(first_path(&[]), None);
}
