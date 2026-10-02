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

mod collapsible {
    use std::{cell::Cell, rc::Rc};

    use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};

    use super::file;
    use crate::{
        changed_files::ChangedFiles,
        theme::{Appearance, set_appearance},
    };

    struct Host {
        reviews: Rc<Cell<usize>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let reviews = self.reviews.clone();
            div().size_full().child(
                ChangedFiles::new("bar", vec![file("src/a.rs", 2, 1), file("b.rs", 1, 0)])
                    .collapsible()
                    .on_review(move |_, _, _| reviews.set(reviews.get() + 1)),
            )
        }
    }

    #[gpui_kit::test]
    fn a_collapsible_list_starts_folded_and_its_header_opens_it(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(true);
        });
        let reviews = Rc::new(Cell::new(0));
        let (_host, cx) = cx.add_window_view(|_, _| Host { reviews: reviews.clone() });
        cx.simulate_resize(size(px(500.), px(400.)));
        cx.run_until_parked();
        assert!(cx.debug_bounds("changed-file-src/a.rs").is_none(), "folded: no rows");
        let review = cx.debug_bounds("changed-files-review").expect("Review is in the header");
        cx.simulate_click(review.center(), gpui_kit::Modifiers::default());
        cx.run_until_parked();
        assert_eq!(reviews.get(), 1);
        assert!(cx.debug_bounds("changed-file-src/a.rs").is_none(), "Review does not unfold the list");
        let toggle = cx.debug_bounds("changed-files-toggle").expect("the header folds and unfolds");
        cx.simulate_click(toggle.origin + gpui_kit::point(px(8.), px(8.)), gpui_kit::Modifiers::default());
        cx.run_until_parked();
        cx.run_until_parked();
        assert!(cx.debug_bounds("changed-file-src/a.rs").is_some(), "open: the rows show");
    }
}
