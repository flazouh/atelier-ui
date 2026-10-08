use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Entity, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn a_row_is_36_tall_the_list_at_most_256_and_the_spring_is_the_web_layout_one() {
    assert_eq!(ROW_HEIGHT, 36.);
    assert_eq!(MAX_HEIGHT, 256.);
    assert_eq!(PAD, 6.);
    let layout = Spring::LAYOUT;
    assert_eq!((layout.stiffness, layout.damping, layout.mass), (360., 32., 0.6));
}

#[test]
fn a_row_out_of_sight_is_scrolled_just_into_it() {
    assert_eq!(scroll_to_show(0., 100., 10., 46.), None, "in sight");
    assert_eq!(scroll_to_show(0., 100., 90., 126.), Some(26.), "below: its bottom meets the view's");
    assert_eq!(scroll_to_show(200., 100., 150., 186.), Some(150.), "above: its top meets the view's");
}

#[test]
fn the_check_grows_from_082_and_fades_in() {
    assert_eq!(check_at(0.), (0.82, 0.));
    assert_eq!(check_at(1.), (1., 1.));
}

struct Page {
    active: Option<usize>,
    rows: usize,
    log: Rc<RefCell<Vec<String>>>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let (pick, hover, this) = (self.log.clone(), self.log.clone(), cx.entity());
        let mut entries = vec![ComboEntry::Group("Folders".into())];
        entries.extend((0..self.rows).map(|i| ComboRow::new(format!("row {i}")).selected(i == 1).debug_name(format!("row-{i}")).into()));
        div().p(px(20.)).flex().items_start().child(
            div().w(px(240.)).child(
                ComboList::new("list", entries)
                    .active(self.active)
                    .empty("Nothing here")
                    .debug_name("list")
                    .on_pick(move |i, _, _| pick.borrow_mut().push(format!("pick {i}")))
                    .on_hover(move |i, _, cx| {
                        hover.borrow_mut().push(format!("hover {i}"));
                        this.update(cx, |p, cx| {
                            p.active = Some(i);
                            cx.notify();
                        });
                    }),
            ),
        )
    }
}

fn open(rows: usize, active: Option<usize>, cx: &mut TestAppContext) -> (Entity<Page>, &mut VisualTestContext, Rc<RefCell<Vec<String>>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let l = log.clone();
    let (page, cx) = cx.add_window_view(move |_, _| Page { active, rows, log: l });
    settle(&page, cx);
    (page, cx, log)
}

fn settle(page: &Entity<Page>, cx: &mut VisualTestContext) {
    for _ in 0..5 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
}

#[gpui_kit::test]
fn a_row_is_36_tall_and_a_click_picks_it_with_its_index(cx: &mut TestAppContext) {
    let (_, cx, log) = open(3, None, cx);
    let row = cx.debug_bounds("row-0").unwrap();
    assert_eq!(f32::from(row.size.height), ROW_HEIGHT);
    let at = cx.debug_bounds("row-2").unwrap().center();
    cx.simulate_click(at, Modifiers::default());
    assert!(log.borrow().contains(&"pick 3".to_string()), "index 3: the heading is entry 0");
}

#[gpui_kit::test]
fn the_pointer_over_a_row_reports_it_and_the_pill_lies_under_the_active_row(cx: &mut TestAppContext) {
    let (page, cx, log) = open(4, Some(1), cx);
    let under = |cx: &mut VisualTestContext, row: &'static str| {
        let (pill, row) = (cx.debug_bounds("combo-pill").unwrap(), cx.debug_bounds(row).unwrap());
        (pill.origin.y == row.origin.y, pill.size.height == row.size.height)
    };
    assert_eq!(under(cx, "row-0"), (true, true), "entry 1 is the first row");
    let at = cx.debug_bounds("row-2").unwrap().center();
    cx.simulate_mouse_move(at, None, Modifiers::default());
    settle(&page, cx);
    assert!(log.borrow().iter().any(|l| l == "hover 3"));
    assert_eq!(page.read_with(cx, |p, _| p.active), Some(3));
    assert_eq!(under(cx, "row-2"), (true, true), "the pill moved to the row under the pointer");
}

#[gpui_kit::test]
fn a_list_with_no_row_says_so(cx: &mut TestAppContext) {
    let (_, cx, _) = open(0, None, cx);
    assert!(cx.debug_bounds("row-0").is_none());
    assert!(cx.debug_bounds("list").is_some());
}

#[gpui_kit::test]
fn the_list_stops_at_256_and_scrolls(cx: &mut TestAppContext) {
    let (_, cx, _) = open(30, None, cx);
    assert_eq!(f32::from(cx.debug_bounds("list").unwrap().size.height), MAX_HEIGHT);
}

/// The folder picker and the task pickers keep the rows they had before the list part: 32 and 28 tall, the
/// active row one step from the panel toward the ink, and the smaller corner.
#[test]
fn the_folder_and_task_styles_keep_the_old_rows() {
    assert_eq!((ComboStyle::Folder.row_x(), ComboStyle::Folder.row_y(), LINE + 2. * ComboStyle::Folder.row_y()), (10., 6., 32.));
    assert_eq!((ComboStyle::Task.row_x(), ComboStyle::Task.row_y(), LINE + 2. * ComboStyle::Task.row_y()), (10., 4., 28.), "10 across, as a select row");
    let theme = crate::theme::Theme::light();
    for style in [ComboStyle::Folder, ComboStyle::Task] {
        let panel = crate::design_preview::panel_fill(&theme, crate::design_preview::elevation(), theme.popover);
        assert_eq!(style.fill(&theme), crate::design_preview::row_tone(&theme, panel), "the pill comes from the panel itself");
        assert_eq!(style.radius(), crate::theme::radius::md());
    }
}

#[test]
fn rows_have_2px_between_them() {
    assert_eq!(ROW_GAP_BETWEEN, 2.);
    assert_eq!((ComboStyle::Task.row_x(), ComboStyle::Task.row_y()), (10., 4.), "the task pickers' rows match a select's");
}
