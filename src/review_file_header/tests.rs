use gpui_kit::{
    Entity, IntoElement, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window,
    div, px,
};

use super::*;
use crate::theme::{Appearance, set_appearance};

struct Page {
    shown: bool,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        div().w(px(700.)).child(
            ReviewFileHeader::new(
                "h",
                "crates/beui/src/menu.rs",
                3,
                1,
                ReviewHandlers::default(),
            )
            .path_shown(self.shown),
        )
    }
}

fn open(shown: bool, cx: &mut TestAppContext) -> (Entity<Page>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let (page, cx) = cx.add_window_view(move |_, _| Page { shown });
    for _ in 0..4 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    (page, cx)
}

#[gpui_kit::test]
fn the_head_shows_the_path_by_default(cx: &mut TestAppContext) {
    let (_, cx) = open(true, cx);
    assert!(cx.debug_bounds("file-header-path").is_some());
}

#[gpui_kit::test]
fn with_the_path_off_the_head_keeps_only_the_counts_and_the_buttons(cx: &mut TestAppContext) {
    let (_, cx) = open(false, cx);
    assert!(
        cx.debug_bounds("file-header-path").is_none(),
        "the breadcrumb is the one path"
    );
}

/// The comment control of the head: a Tab stop that starts a comment on the caret's line, as the `c` key does.
#[gpui_kit::test]
fn the_head_has_a_comment_control_that_a_tab_reaches_and_a_press_runs(cx: &mut TestAppContext) {
    use std::{cell::Cell, rc::Rc};
    struct Page(Rc<Cell<usize>>);
    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
            let count = self.0.clone();
            let handlers =
                ReviewHandlers::default().on_comment(move |_, _| count.set(count.get() + 1));
            div()
                .w(px(700.))
                .child(ReviewFileHeader::new("h", "src/lib.rs", 3, 1, handlers).path_shown(false))
        }
    }
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let count = Rc::new(Cell::new(0));
    let c = count.clone();
    let (page, cx) = cx.add_window_view(move |_, _| Page(c));
    for _ in 0..4 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    let key = cx
        .debug_bounds("Comment")
        .expect("the head has a Comment control")
        .center();
    // Tab reaches it: it is the only live control, so the first Tab lands on it, and Enter presses it.
    cx.update(|window, cx| window.focus_next(cx));
    assert!(
        cx.update(|window, cx| window.focused(cx)).is_some(),
        "Tab landed on something"
    );
    // The element hears the key once a frame has drawn it focused.
    for _ in 0..2 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    cx.simulate_keystrokes("enter");
    cx.simulate_event(gpui_kit::KeyUpEvent {
        keystroke: gpui_kit::Keystroke::parse("enter").unwrap(),
    });
    cx.run_until_parked();
    assert_eq!(count.get(), 1, "Tab then Enter ran the comment handler");
    cx.simulate_click(key, gpui_kit::Modifiers::default());
    assert_eq!(count.get(), 2);
}
