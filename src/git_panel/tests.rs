use gpui_kit::{Context, ParentElement, Render, Styled, TestAppContext, div, size};

use super::*;
use crate::scale::px;

#[test]
fn each_change_has_its_letter_and_colour() {
    let theme = Theme::dark();
    assert_eq!(letter(&FileChange::Modified), "M");
    assert_eq!(letter(&FileChange::Added), "A");
    assert_eq!(letter(&FileChange::Deleted), "D");
    assert_eq!(letter(&FileChange::Renamed { from: "old.rs".into() }), "R");
    assert_eq!(tone(&FileChange::Modified, &theme), theme.info);
    assert_eq!(tone(&FileChange::Added, &theme), theme.success);
    assert_eq!(tone(&FileChange::Deleted, &theme), theme.danger);
}

struct View {
    opened: std::rc::Rc<std::cell::RefCell<Vec<SharedString>>>,
    session: bool,
}

impl Render for View {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let opened = self.opened.clone();
        let files = vec![ChangedFile::new("src/lib.rs", 3, 1), ChangedFile::new("README.md", 2, 0).change(FileChange::Added)];
        div().size_full().child(
            GitPanel::new("git", "atelier")
                .branch(Some("ui-batch".into()))
                .session(self.session.then(|| "Fix the rail".into()))
                .files(if self.session { files } else { Vec::new() })
                .current(Some("README.md".into()))
                .on_open(move |path, _, _| opened.borrow_mut().push(path.clone())),
        )
    }
}

#[gpui_kit::test]
fn it_shows_the_repo_the_branch_and_the_files_and_a_press_opens_one(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let opened = std::rc::Rc::default();
    let shared = std::rc::Rc::clone(&opened);
    let (_, cx) = cx.add_window_view(move |_, _| View { opened: shared, session: true });
    cx.simulate_resize(size(px(300.), px(500.)));
    cx.run_until_parked();
    for selector in ["git-repo", "git-branch", "git-session", "git-file-src/lib.rs", "git-file-README.md"] {
        assert!(cx.debug_bounds(selector).is_some(), "{selector} is drawn");
    }
    let row = cx.debug_bounds("git-file-src/lib.rs").unwrap();
    cx.simulate_click(row.center(), gpui_kit::Modifiers::default());
    assert_eq!(*opened.borrow(), vec![SharedString::from("src/lib.rs")]);
}

#[gpui_kit::test]
fn with_no_session_in_focus_it_lists_no_file(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let (_, cx) = cx.add_window_view(|_, _| View { opened: std::rc::Rc::default(), session: false });
    cx.run_until_parked();
    assert!(cx.debug_bounds("git-repo").is_some());
    assert!(cx.debug_bounds("git-session").is_none() && cx.debug_bounds("git-file-src/lib.rs").is_none());
}

struct WithTrees(bool);

impl Render for WithTrees {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let rows = if self.0 { crate::worktree_list::tests::rows() } else { Vec::new() };
        div().size_full().child(GitPanel::new("git", "atelier").branch(Some("main".into())).worktrees(rows))
    }
}

#[gpui_kit::test]
fn the_worktrees_show_under_the_changes_and_not_at_all_when_there_are_none(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let (_, cx) = cx.add_window_view(|_, _| WithTrees(true));
    cx.simulate_resize(size(px(300.), px(600.)));
    cx.run_until_parked();
    let trees = cx.debug_bounds("worktrees").expect("listed");
    assert!(trees.top() > cx.debug_bounds("git-branch").unwrap().bottom());
    assert!(cx.debug_bounds("worktree-/r/atelier-fix").is_some());
    let (_, cx) = cx.add_window_view(|_, _| WithTrees(false));
    cx.run_until_parked();
    assert!(cx.debug_bounds("worktrees").is_none());
}
