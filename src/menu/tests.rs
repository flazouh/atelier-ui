use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Entity, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn the_clip_starts_a_16px_square_at_the_origin_and_ends_as_the_whole_panel() {
    let start = collapsed((100., 50.), (224., 120.));
    assert_eq!(start, Inset { top: 42., right: 116., bottom: 62., left: 92. });
    let (all, corner) = unfolded(start, 1.);
    assert_eq!(all, Inset { top: 0., right: 0., bottom: 0., left: 0. });
    assert_eq!(corner, 12.);
    let (first, corner) = unfolded(start, 0.);
    assert_eq!((first, corner), (start, 10.));
    let (half, _) = unfolded(start, 0.5);
    assert_eq!(half.left, 46.);
}

#[test]
fn an_origin_keeps_12px_from_the_edges() {
    assert_eq!(Origin::At(0., 0.).point((224., 120.)), (12., 12.));
    assert_eq!(Origin::TopRight.point((224., 120.)), (212., 12.));
    assert_eq!(Origin::BottomLeft.point((224., 120.)), (12., 108.));
    assert_eq!(Origin::At(50., 60.).point((20., 20.)), (12., 12.), "a panel too small to keep clear takes 12");
}

#[test]
fn up_and_down_wrap_and_skip_the_rows_that_cannot_be_reached() {
    let rows = [1, 3, 4];
    assert_eq!(walk(&rows, None, 1), Some(1));
    assert_eq!(walk(&rows, Some(1), 1), Some(3));
    assert_eq!(walk(&rows, Some(4), 1), Some(1), "down from the last wraps to the first");
    assert_eq!(walk(&rows, Some(1), -1), Some(4), "up from the first wraps to the last");
    assert_eq!(walk(&[], Some(1), 1), None);
}

#[test]
fn typing_jumps_to_the_first_row_whose_words_start_with_the_letters() {
    let rows = vec![(0, "Copy".to_string()), (2, "Cut".to_string()), (3, " Paste".to_string())];
    assert_eq!(jump(&rows, "c"), Some(0));
    assert_eq!(jump(&rows, "cu"), Some(2));
    assert_eq!(jump(&rows, "P"), Some(3), "case and leading space do not matter");
    assert_eq!(jump(&rows, "z"), None);
}

struct Host {
    log: Rc<RefCell<Vec<String>>>,
    origin: Option<Origin>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let log = |name: &'static str| {
            let log = self.log.clone();
            move |_: &mut Window, _: &mut gpui_kit::App| log.borrow_mut().push(name.to_string())
        };
        let dismiss = log("dismiss");
        let mut menu = Menu::new(
            "menu",
            [
                Entry::Label("Edit".into()),
                MenuItem::new("Copy").shortcut("⌘C").debug_name("row-copy").on_select(log("copy")).into(),
                MenuItem::new("Cut").debug_name("row-cut").disabled(true).on_select(log("cut")).into(),
                Entry::Separator,
                MenuItem::new("Paste").debug_name("row-paste").on_select(log("paste")).into(),
                MenuItem::new("Delete").tone(Tone::Destructive).debug_name("row-delete").on_select(log("delete")).into(),
                MenuItem::new("Wrap lines").choice(Choice::Check(true)).close_on_select(false).debug_name("row-wrap").on_select(log("wrap")).into(),
            ],
        )
        .debug_name("panel")
        .on_dismiss(dismiss);
        if let Some(origin) = self.origin {
            menu = menu.origin(origin);
        }
        div().p(px(40.)).flex().child(menu)
    }
}

fn open(origin: Option<Origin>, reduce: bool, cx: &mut TestAppContext) -> (Entity<Host>, &mut VisualTestContext, Rc<RefCell<Vec<String>>>) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let l = log.clone();
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
    });
    let (host, cx) = cx.add_window_view(move |_, _| Host { log: l, origin });
    for _ in 0..4 {
        cx.run_until_parked();
        host.update(cx, |_, cx| cx.notify());
    }
    (host, cx, log)
}

fn at(cx: &mut VisualTestContext, name: &'static str) -> gpui_kit::Point<Pixels> {
    cx.debug_bounds(name).unwrap_or_else(|| panic!("no {name}")).center()
}

#[gpui_kit::test]
fn the_panel_is_at_least_224_wide_and_the_first_reachable_row_has_focus(cx: &mut TestAppContext) {
    let (_, cx, log) = open(None, true, cx);
    let panel = cx.debug_bounds("panel").unwrap();
    assert!(f32::from(panel.size.width) >= MenuLook::BAR.min_width);
    cx.simulate_keystrokes("enter");
    assert_eq!(*log.borrow(), vec!["dismiss", "copy"], "Enter chose the first row, which held focus");
}

#[gpui_kit::test]
fn down_skips_a_disabled_row_and_wraps_and_the_keys_choose(cx: &mut TestAppContext) {
    let (_, cx, log) = open(None, true, cx);
    cx.simulate_keystrokes("down enter");
    assert_eq!(*log.borrow(), vec!["dismiss", "paste"], "Cut is disabled");
    log.borrow_mut().clear();
    cx.simulate_keystrokes("up up enter");
    assert_eq!(*log.borrow(), vec!["wrap"], "up from Paste to Copy, up again wraps to the last row, which keeps the menu open");
}

#[gpui_kit::test]
fn end_and_home_go_to_the_ends_and_typing_jumps(cx: &mut TestAppContext) {
    let (_, cx, log) = open(None, true, cx);
    cx.simulate_keystrokes("end space");
    assert_eq!(*log.borrow(), vec!["wrap"]);
    log.borrow_mut().clear();
    cx.simulate_keystrokes("home d enter");
    assert_eq!(*log.borrow(), vec!["dismiss", "delete"], "d jumped to Delete");
}

#[gpui_kit::test]
fn a_click_chooses_a_row_and_a_disabled_row_does_nothing(cx: &mut TestAppContext) {
    let (_, cx, log) = open(None, true, cx);
    let cut = at(cx, "row-cut");
    cx.simulate_click(cut, Modifiers::default());
    assert!(log.borrow().is_empty());
    let paste = at(cx, "row-paste");
    cx.simulate_click(paste, Modifiers::default());
    assert_eq!(*log.borrow(), vec!["dismiss", "paste"]);
}

#[gpui_kit::test]
fn the_pointer_over_a_row_makes_it_the_active_one(cx: &mut TestAppContext) {
    let (host, cx, log) = open(None, true, cx);
    let delete = at(cx, "row-delete");
    cx.simulate_mouse_move(delete, None, Modifiers::default());
    host.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    assert_eq!(*log.borrow(), vec!["dismiss", "delete"]);
}

#[gpui_kit::test]
fn with_an_origin_the_panel_is_invisible_at_first_and_whole_once_unfolded(cx: &mut TestAppContext) {
    let (host, cx, _) = open(Some(Origin::TopRight), false, cx);
    let full = cx.debug_bounds("panel").expect("measured after the first frames");
    crate::motion::clock::freeze();
    crate::motion::clock::advance(std::time::Duration::from_millis(400));
    for _ in 0..3 {
        host.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
    }
    let done = cx.debug_bounds("panel").unwrap();
    assert_eq!((done.size.width, done.size.height), (full.size.width, full.size.height));
}

/// The panel keeps one size from its first frame to its last: it does not shrink a border at a time while it
/// unfolds, nor jump when the unfold ends.
#[gpui_kit::test]
fn the_panel_keeps_its_size_through_the_unfold(cx: &mut TestAppContext) {
    let (host, cx, _) = open(Some(Origin::TopRight), false, cx);
    let first = cx.debug_bounds("panel").expect("measured after the first frames");
    crate::motion::clock::freeze();
    for step in 0..40 {
        crate::motion::clock::advance(std::time::Duration::from_millis(16));
        host.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        let now = cx.debug_bounds("panel").unwrap();
        assert_eq!((now.size.width, now.size.height), (first.size.width, first.size.height), "size at step {step}");
    }
}

#[test]
fn the_panel_size_is_the_probe_plus_its_border() {
    assert_eq!(panel_size(gpui_kit::px(273.), gpui_kit::px(194.)), (275., 196.));
}
#[test]
fn the_panel_size_is_in_design_pixels_and_its_border_stays_one_real_pixel() {
    crate::scale::set_zoom(1.5);
    let (w, h) = panel_size(gpui_kit::px(298.), gpui_kit::px(148.));
    crate::scale::set_zoom(1.);
    assert!((w - 200.).abs() < 0.01, "300 window pixels at 1.5 are 200 design pixels: {w}");
    assert!((h - 100.).abs() < 0.01, "150 window pixels at 1.5 are 100 design pixels: {h}");
}

/// Each site keeps the menu it had before the Menu part: the numbers of the old review bar, merge, prompt and
/// project menus. Rows are 32, 30, 36 and 28 tall; no panel has a border.
#[test]
fn each_site_keeps_the_menu_it_had() {
    let row = |look: MenuLook| LINE + 2. * look.row_y;
    assert_eq!((row(MenuLook::BAR), row(MenuLook::MERGE), row(MenuLook::PROMPT), row(MenuLook::PROJECT)), (32., 30., 36., 28.));
    assert_eq!((MenuLook::BAR.min_width, MenuLook::MERGE.min_width, MenuLook::PROMPT.min_width, MenuLook::PROJECT.min_width), (180., 240., 224., 180.));
    assert_eq!(height_in(MenuLook::MERGE, 3), 2. * 4. + 3. * 30. + 2., "padding, rows and the edge");
    assert_eq!(height_in(MenuLook::PROMPT, 2), 2. * 6. + 2. * 36. + 2.);
    assert_eq!((MenuLook::PROJECT.panel_radius, MenuLook::PROJECT.row_radius), (8., 6.));
    assert_eq!((MenuLook::BAR.panel_radius, MenuLook::BAR.row_radius), (12., 8.));
    assert_eq!(MenuLook::PROMPT.shadow, 1.4, "the prompt menu had a stronger shadow");
}

/// The fill is whole early in the unfold, so the text under the panel never shows through its rows.
#[test]
fn the_fill_is_whole_a_sixth_of_the_way_into_the_unfold() {
    assert_eq!(fill_opacity(0.), 0.);
    assert!(fill_opacity(0.05) > 0. && fill_opacity(0.05) < 1.);
    assert_eq!(fill_opacity(1. / 6.), 1.);
    assert_eq!(fill_opacity(0.5), 1.);
    assert_eq!(fill_opacity(1.), 1.);
}

struct SubHost {
    log: Rc<RefCell<Vec<String>>>,
}

impl Render for SubHost {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let log = |name: &'static str| {
            let log = self.log.clone();
            move |_: &mut Window, _: &mut gpui_kit::App| log.borrow_mut().push(name.to_string())
        };
        let menu = Menu::new(
            "menu",
            [
                MenuItem::new("Plain").debug_name("row-plain").on_select(log("plain")).into(),
                MenuItem::new("Hand off")
                    .debug_name("row-handoff")
                    .submenu([
                        MenuItem::new("Claude").debug_name("row-claude").on_select(log("claude")).into(),
                        MenuItem::new("Codex").debug_name("row-codex").on_select(log("codex")).into(),
                    ])
                    .into(),
            ],
        )
        .on_dismiss(log("dismiss"));
        div().p(px(40.)).flex().child(menu)
    }
}

fn open_with_submenu(cx: &mut TestAppContext) -> (&mut VisualTestContext, Rc<RefCell<Vec<String>>>) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let l = log.clone();
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let (host, cx) = cx.add_window_view(move |_, _| SubHost { log: l });
    for _ in 0..4 {
        cx.run_until_parked();
        host.update(cx, |_, cx| cx.notify());
    }
    (cx, log)
}

fn settle(cx: &mut VisualTestContext) {
    for _ in 0..4 {
        cx.run_until_parked();
        cx.update(|window, _| window.refresh());
    }
}

#[gpui_kit::test]
fn a_row_with_a_submenu_opens_it_beside_itself_on_hover(cx: &mut TestAppContext) {
    let (cx, _) = open_with_submenu(cx);
    assert!(cx.debug_bounds("row-claude").is_none(), "closed at first");
    let at = at(cx, "row-handoff");
    cx.simulate_mouse_move(at, None, Modifiers::default());
    settle(cx);
    let row = cx.debug_bounds("row-handoff").unwrap();
    let claude = cx.debug_bounds("row-claude").expect("the submenu is open");
    assert!(claude.left() >= row.right(), "it sits beside the row, not over it");
}

#[gpui_kit::test]
fn a_row_of_the_submenu_chooses_and_the_row_that_opened_it_chooses_nothing(cx: &mut TestAppContext) {
    let (cx, log) = open_with_submenu(cx);
    let handoff = at(cx, "row-handoff");
    cx.simulate_click(handoff, Modifiers::default());
    settle(cx);
    assert!(log.borrow().is_empty(), "opening a submenu is not a choice");
    let codex = at(cx, "row-codex");
    cx.simulate_click(codex, Modifiers::default());
    assert_eq!(*log.borrow(), vec!["dismiss", "codex"]);
}

#[gpui_kit::test]
fn hovering_another_row_closes_the_submenu(cx: &mut TestAppContext) {
    let (cx, _) = open_with_submenu(cx);
    let handoff = at(cx, "row-handoff");
    cx.simulate_mouse_move(handoff, None, Modifiers::default());
    settle(cx);
    assert!(cx.debug_bounds("row-claude").is_some());
    let plain = at(cx, "row-plain");
    cx.simulate_mouse_move(plain, None, Modifiers::default());
    settle(cx);
    assert!(cx.debug_bounds("row-claude").is_none());
}

#[gpui_kit::test]
fn right_opens_the_submenu_and_left_and_escape_close_only_it(cx: &mut TestAppContext) {
    let (cx, log) = open_with_submenu(cx);
    cx.simulate_keystrokes("down right");
    settle(cx);
    assert!(cx.debug_bounds("row-claude").is_some(), "Right opened it");
    cx.simulate_keystrokes("left");
    settle(cx);
    assert!(cx.debug_bounds("row-claude").is_none(), "Left closed it");
    cx.simulate_keystrokes("right escape");
    settle(cx);
    assert!(cx.debug_bounds("row-claude").is_none(), "Escape closed it too");
    cx.simulate_keystrokes("right down enter");
    settle(cx);
    assert_eq!(*log.borrow(), vec!["dismiss", "codex"], "the keys walk the submenu and choose in it");
}

#[test]
fn a_tree_of_branches_becomes_rows_that_open_menus_and_leaves_that_pick() {
    let tree = vec![
        Branch::leaf("plain", "Plain"),
        Branch::with("agent", "Agent", vec![Branch::leaf("agent/a", "A"), Branch::leaf("agent/b", "B")]),
    ];
    let heard = Rc::new(RefCell::new(Vec::new()));
    let sink = heard.clone();
    let pick: Pick = Rc::new(move |id, _, _| sink.borrow_mut().push(id.to_string()));
    let rows = entries_of(&tree, &pick);
    assert_eq!(rows.len(), 2);
    let Entry::Item(agent) = &rows[1] else { panic!("a row") };
    assert_eq!(agent.submenu.as_ref().map(Vec::len), Some(2), "a branch holds its branches");
    assert!(agent.on_select.is_none(), "and chooses nothing itself");
    let Entry::Item(plain) = &rows[0] else { panic!("a row") };
    assert!(plain.submenu.is_none() && plain.on_select.is_some(), "a leaf picks");
}

struct LeadHost;

impl Render for LeadHost {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        use crate::model_badge::BrandMark;
        let entries: Vec<Entry> = vec![
            MenuItem::new("Claude Code").lead(Lead::Mark(BrandMark::new("a-light.svg", "a-dark.svg"))).into(),
            MenuItem::new("Codex").lead(Lead::of(None)).into(),
            MenuItem::new("Plain").into(),
        ];
        div().p(px(40.)).flex().child(Menu::new("lead-menu", entries))
    }
}

#[gpui_kit::test]
fn a_row_with_a_lead_shows_it_before_its_words_a_mark_or_the_first_letter(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let (host, cx) = cx.add_window_view(|_, _| LeadHost);
    for _ in 0..4 {
        cx.run_until_parked();
        host.update(cx, |_, cx| cx.notify());
    }
    assert!(cx.debug_bounds("menu-lead-Claude Code").is_some(), "the mark's slot");
    assert!(cx.debug_bounds("select-monogram-C").is_some(), "a row without a mark gets its first letter");
    assert!(cx.debug_bounds("menu-lead-Plain").is_none(), "a row with no lead gets nothing");
}

/// At a zoom the panel is as wide as the zoom makes it, and keeps that size through the unfold: its border is one real
/// pixel at any zoom, so a panel sized in design pixels with the border counted twice would grow on each frame.
#[gpui_kit::test]
fn at_a_zoom_the_panel_keeps_its_size_through_the_unfold(cx: &mut TestAppContext) {
    crate::scale::set_zoom(1.5);
    let (host, cx, _) = open(Some(Origin::TopRight), false, cx);
    let first = cx.debug_bounds("panel").expect("measured after the first frames");
    crate::motion::clock::freeze();
    for step in 0..40 {
        crate::motion::clock::advance(std::time::Duration::from_millis(16));
        host.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        let now = cx.debug_bounds("panel").unwrap();
        assert!((f32::from(now.size.width) - f32::from(first.size.width)).abs() < 0.6, "width at step {step}: {:?} then {:?}", first.size, now.size);
        assert!((f32::from(now.size.height) - f32::from(first.size.height)).abs() < 0.6, "height at step {step}");
    }
    crate::scale::set_zoom(1.);
}
