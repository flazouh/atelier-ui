use super::*;

fn tree() -> FileTree {
    FileTree::new(&[ChangedFile::new("src/a.rs", 1, 0), ChangedFile::new("src/b.rs", 1, 0), ChangedFile::new("README.md", 1, 0)])
}

/// Presses `keys` from `cursor`, and returns where the cursor lands, what is folded, and what opened.
fn press(cursor: Option<&str>, keys: &[TreeKey]) -> (Option<String>, Vec<String>, Option<String>) {
    let tree = tree();
    let (mut cursor, mut folded) = (cursor.map(SharedString::from), HashSet::new());
    let mut opened = None;
    for k in keys {
        opened = key(&tree.rows(&folded), &mut cursor, &mut folded, *k).or(opened);
    }
    let mut folded: Vec<String> = folded.into_iter().map(|f| f.to_string()).collect();
    folded.sort();
    (cursor.map(|c| c.to_string()), folded, opened.map(|o| o.to_string()))
}

#[test]
fn up_and_down_walk_the_rows_and_stop_at_the_ends() {
    // Rows: src, src/a.rs, src/b.rs, README.md.
    assert_eq!(press(None, &[TreeKey::Down]).0.as_deref(), Some("src"));
    assert_eq!(press(Some("src"), &[TreeKey::Down, TreeKey::Down]).0.as_deref(), Some("src/b.rs"));
    assert_eq!(press(Some("README.md"), &[TreeKey::Down]).0.as_deref(), Some("README.md"));
    assert_eq!(press(Some("src"), &[TreeKey::Up]).0.as_deref(), Some("src"));
}

#[test]
fn left_folds_a_folder_then_goes_to_the_parent_and_right_undoes_it() {
    assert_eq!(press(Some("src"), &[TreeKey::Left]).1, ["src"]);
    assert_eq!(press(Some("src/b.rs"), &[TreeKey::Left]).0.as_deref(), Some("src"));
    let (cursor, folded, _) = press(Some("src"), &[TreeKey::Left, TreeKey::Right]);
    assert_eq!((cursor.as_deref(), folded.len()), (Some("src"), 0));
    assert_eq!(press(Some("src"), &[TreeKey::Right]).0.as_deref(), Some("src/a.rs"));
}

#[test]
fn a_folded_folder_hides_its_rows_from_the_keys() {
    assert_eq!(press(Some("src"), &[TreeKey::Left, TreeKey::Down]).0.as_deref(), Some("README.md"));
}

#[test]
fn enter_opens_a_file_and_folds_a_folder() {
    assert_eq!(press(Some("src/a.rs"), &[TreeKey::Enter]).2.as_deref(), Some("src/a.rs"));
    let (_, folded, opened) = press(Some("src"), &[TreeKey::Enter]);
    assert_eq!((folded, opened), (vec!["src".to_string()], None));
}

mod ring {
    use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};

    use super::super::{ChangedFile, ChangedFileTree};
    use crate::theme::{Appearance, set_appearance};

    struct Host {
        focus: gpui_kit::FocusHandle,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let files = vec![ChangedFile::new("src/a.rs", 1, 0), ChangedFile::new("README.md", 1, 0)];
            div().w(px(300.)).h(px(300.)).child(ChangedFileTree::new("tree", files).track_focus(&self.focus))
        }
    }

    #[gpui_kit::test]
    fn the_row_under_the_keyboard_cursor_wears_the_shared_ring_and_only_that_row(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            super::super::bind_keys(cx);
            set_appearance(Appearance::Light, cx);
            cx.set_reduce_motion(true);
        });
        let (host, cx) = cx.add_window_view(|_, cx| Host { focus: cx.focus_handle() });
        cx.simulate_resize(size(px(400.), px(400.)));
        cx.run_until_parked();
        assert!(cx.debug_bounds("row-ring").is_none(), "no focus, no cursor, no ring");
        let focus = host.read_with(cx, |h, _| h.focus.clone());
        cx.update(|window, cx| focus.focus(window, cx));
        cx.simulate_keystrokes("down");
        for _ in 0..3 {
            host.update(cx, |_, cx| cx.notify());
            cx.run_until_parked();
        }
        assert!(cx.debug_bounds("row-ring").is_some(), "the cursor row has the ring");
    }
}

#[test]
fn opening_a_folder_opens_it_and_the_folders_it_sits_in_and_no_others() {
    use crate::changed_file_tree::unfolds;
    assert!(unfolds("crates", "crates/beui/src"), "an ancestor");
    assert!(unfolds("crates/beui/src", "crates/beui/src"), "itself");
    assert!(!unfolds("crates/beui/src/menu", "crates/beui/src"), "a folder inside it stays as it is");
    assert!(!unfolds("crates2", "crates/beui"), "a name that only starts the same");
}
