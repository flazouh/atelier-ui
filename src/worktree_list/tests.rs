use gpui_kit::{Context, ParentElement, Render, Styled, TestAppContext, div, size};

use super::*;
use crate::scale::px;

#[test]
fn each_tone_has_its_colour() {
    let theme = Theme::dark();
    assert_eq!(note_colour(NoteTone::Quiet, &theme), theme.muted_foreground);
    assert_eq!(note_colour(NoteTone::Warning, &theme), theme.warning);
    assert_eq!(note_colour(NoteTone::Good, &theme), theme.success);
    assert_eq!(sessions_words(0), None);
    assert_eq!(sessions_words(1).as_deref(), Some("1 session"));
    assert_eq!(sessions_words(3).as_deref(), Some("3 sessions"));
}

pub(crate) fn rows() -> Vec<WorktreeRow> {
    vec![
        WorktreeRow {
            path: "/r/atelier".into(),
            folder: "~/code/atelier".into(),
            branch: Some("main".into()),
            main: true,
            notes: vec![],
            sessions: 2,
        },
        WorktreeRow {
            path: "/r/atelier-fix".into(),
            folder: "~/code/atelier-fix".into(),
            branch: Some("fix".into()),
            main: false,
            notes: vec![WorktreeNote::new("3 uncommitted", NoteTone::Warning), WorktreeNote::new("2 only here", NoteTone::Warning)],
            sessions: 0,
        },
        WorktreeRow { path: "/r/loose".into(), folder: "~/code/loose".into(), branch: None, main: false, notes: vec![WorktreeNote::new("merged", NoteTone::Good)], sessions: 0 },
    ]
}

struct View;

impl Render for View {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(WorktreeList::new("trees", rows()))
    }
}

#[gpui_kit::test]
fn each_worktree_has_a_row_and_only_the_main_checkout_is_marked(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let (_, cx) = cx.add_window_view(|_, _| View);
    cx.simulate_resize(size(px(300.), px(500.)));
    cx.run_until_parked();
    let tops: Vec<f32> = ["worktree-/r/atelier", "worktree-/r/atelier-fix", "worktree-/r/loose"]
        .iter()
        .map(|s| f32::from(cx.debug_bounds(s).unwrap_or_else(|| panic!("{s} is drawn")).top()))
        .collect();
    assert!(tops.windows(2).all(|w| w[0] < w[1]), "in the order given: {tops:?}");
    assert!(cx.debug_bounds("worktree-main-/r/atelier").is_some());
    assert!(cx.debug_bounds("worktree-main-/r/atelier-fix").is_none());
}
