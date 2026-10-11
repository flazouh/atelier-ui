use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Context, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use super::{NavGroup, NavList, NavRow};
use crate::theme::{Appearance, set_appearance};

struct Host {
    open: Vec<&'static str>,
    selected: &'static str,
    flat: bool,
    log: Rc<RefCell<Vec<String>>>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (picks, toggles) = (self.log.clone(), self.log.clone());
        let groups = if self.flat {
            vec![NavGroup::flat([NavRow::new("buttons", "Buttons").note("Controls"), NavRow::new("colors", "Colors").note("Foundations")])]
        } else {
            vec![
                NavGroup::new("Foundations", [NavRow::new("colors", "Colors"), NavRow::new("icons", "Icons")]).open(self.open.contains(&"Foundations")),
                NavGroup::new("Controls", [NavRow::new("buttons", "Buttons")]).open(self.open.contains(&"Controls")),
            ]
        };
        div().w(px(220.)).child(
            NavList::new("nav")
                .groups(groups)
                .selected(self.selected)
                .on_pick(move |key, _, _| picks.borrow_mut().push(format!("pick {key}")))
                .on_toggle(move |name, _, _| toggles.borrow_mut().push(format!("toggle {name}"))),
        )
    }
}

fn open(cx: &mut TestAppContext, host: impl FnOnce(Rc<RefCell<Vec<String>>>) -> Host) -> (Rc<RefCell<Vec<String>>>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let (_, cx) = cx.add_window_view({
        let log = log.clone();
        move |_, _| host(log)
    });
    cx.run_until_parked();
    (log, cx)
}

#[gpui_kit::test]
fn a_folded_group_shows_its_head_and_no_rows_and_an_open_one_shows_its_rows_under_its_head(cx: &mut TestAppContext) {
    let (_, cx) = open(cx, |log| Host { open: vec!["Foundations"], selected: "colors", flat: false, log });
    let head = cx.debug_bounds("nav-group-Foundations").expect("the open group's head is drawn");
    let colors = cx.debug_bounds("nav-row-colors").expect("its first row is drawn");
    let icons = cx.debug_bounds("nav-row-icons").expect("its second row is drawn");
    assert!(head.bottom() <= colors.top() && colors.bottom() <= icons.top(), "the rows stand under the head, in order");
    let folded = cx.debug_bounds("nav-group-Controls").expect("the folded group's head is drawn");
    assert!(icons.bottom() <= folded.top(), "the next group is under the open one's rows");
    assert!(cx.debug_bounds("nav-row-buttons").is_none(), "a folded group shows no row");
}

#[gpui_kit::test]
fn only_the_chosen_row_is_marked(cx: &mut TestAppContext) {
    let (_, cx) = open(cx, |log| Host { open: vec!["Foundations", "Controls"], selected: "icons", flat: false, log });
    let mark = cx.debug_bounds("nav-row-chosen").expect("a row is chosen");
    let icons = cx.debug_bounds("nav-row-icons").expect("the row is drawn");
    assert!(icons.contains(&mark.origin), "the mark is in the chosen row");
}

#[gpui_kit::test]
fn a_press_on_a_head_or_a_row_tells_the_owner_which_and_changes_nothing_by_itself(cx: &mut TestAppContext) {
    let (log, cx) = open(cx, |log| Host { open: vec!["Foundations"], selected: "colors", flat: false, log });
    cx.simulate_click(cx.debug_bounds("nav-group-Controls").unwrap().center(), Modifiers::default());
    cx.simulate_click(cx.debug_bounds("nav-row-icons").unwrap().center(), Modifiers::default());
    cx.run_until_parked();
    assert_eq!(*log.borrow(), ["toggle Controls", "pick icons"]);
    assert!(cx.debug_bounds("nav-row-buttons").is_none(), "the owner did not open the group, so it stays folded");
}

#[gpui_kit::test]
fn a_group_with_no_name_has_no_head_and_always_shows_its_rows_with_their_notes(cx: &mut TestAppContext) {
    let (log, cx) = open(cx, |log| Host { open: vec![], selected: "buttons", flat: true, log });
    assert!(cx.debug_bounds("nav-group-").is_none(), "no head");
    let buttons = cx.debug_bounds("nav-row-buttons").expect("the first result is drawn");
    let colors = cx.debug_bounds("nav-row-colors").expect("the second result is drawn");
    assert!(buttons.bottom() <= colors.top());
    cx.simulate_click(colors.center(), Modifiers::default());
    assert_eq!(*log.borrow(), ["pick colors"]);
}
