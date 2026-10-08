use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    Entity, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext,
    VisualTestContext, Window, div, px,
};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn a_short_path_shows_every_part_and_a_long_one_folds_its_middle() {
    assert_eq!(shown(3, 4), vec![Some(0), Some(1), Some(2)]);
    assert_eq!(shown(4, 4), vec![Some(0), Some(1), Some(2), Some(3)]);
    assert_eq!(
        shown(6, 4),
        vec![Some(0), None, Some(4), Some(5)],
        "the first, the ellipsis, and the last two"
    );
    assert_eq!(shown(6, 3), vec![Some(0), None, Some(5)]);
    assert_eq!(
        shown(6, 1),
        vec![Some(0), None, Some(5)],
        "at least three slots"
    );
    assert_eq!(hidden(6, 4), 1..4);
    assert_eq!(hidden(3, 4), 0..0);
}

struct Page {
    crumbs: Vec<&'static str>,
    log: Rc<RefCell<Vec<usize>>>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let log = self.log.clone();
        div().p(px(20.)).w(px(700.)).child(
            Breadcrumb::new("bc", self.crumbs.iter().map(|c| Crumb::new(*c)))
                .debug_name("bc")
                .on_press(move |i, _, _| log.borrow_mut().push(i)),
        )
    }
}

fn open<'a>(
    crumbs: Vec<&'static str>,
    cx: &'a mut TestAppContext,
) -> (
    Entity<Page>,
    &'a mut VisualTestContext,
    Rc<RefCell<Vec<usize>>>,
) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let l = log.clone();
    let (page, cx) = cx.add_window_view(move |_, _| Page { crumbs, log: l });
    for _ in 0..4 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    (page, cx, log)
}

#[gpui_kit::test]
fn a_press_on_a_folder_reports_its_index_and_the_page_is_not_pressed(cx: &mut TestAppContext) {
    let (_, cx, log) = open(vec!["crates", "beui", "src", "menu.rs"], cx);
    let at = cx.debug_bounds("bc-1").unwrap().center();
    cx.simulate_click(at, Modifiers::default());
    assert_eq!(*log.borrow(), vec![1]);
    let page = cx.debug_bounds("bc-3").unwrap();
    assert_eq!(f32::from(page.size.height), HEIGHT);
    cx.simulate_click(page.center(), Modifiers::default());
    assert_eq!(
        *log.borrow(),
        vec![1],
        "the last part is where the reader is"
    );
}

#[gpui_kit::test]
fn a_long_path_folds_and_the_ellipsis_lists_the_hidden_folders(cx: &mut TestAppContext) {
    let (page, cx, log) = open(
        vec!["crates", "beui", "src", "menu", "tests", "menu.rs"],
        cx,
    );
    assert!(
        cx.debug_bounds("bc-1").is_none() && cx.debug_bounds("bc-3").is_none(),
        "the middle is folded"
    );
    assert!(
        cx.debug_bounds("bc-0").is_some()
            && cx.debug_bounds("bc-4").is_some()
            && cx.debug_bounds("bc-5").is_some()
    );
    let more = cx.debug_bounds("bc-more").expect("the ellipsis");
    cx.simulate_click(more.center(), Modifiers::default());
    for _ in 0..5 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    cx.simulate_keystrokes("down enter");
    for _ in 0..3 {
        cx.run_until_parked();
    }
    assert_eq!(*log.borrow(), vec![2], "the second hidden part, src");
}

/// The path wears the editor header's numbers: 28px like the tabs beside it, 10px across, 14px file icons.
#[test]
fn the_path_is_as_tall_as_a_tab_and_pads_like_one() {
    assert_eq!((HEIGHT, LINK_PAD, ICON), (28., 10., 14.));
}
