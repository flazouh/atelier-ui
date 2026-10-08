use gpui_kit::{Entity, TestAppContext, VisualTestContext, size};

use super::*;
use crate::theme::{Appearance, set_appearance};

fn teams() -> Vec<MultiOption> {
    vec![
        MultiOption::new("design", "Design").group("Product teams"),
        MultiOption::new("engineering", "Engineering")
            .group("Product teams")
            .keywords(["code", "build"]),
        MultiOption::new("product", "Product")
            .group("Product teams")
            .disabled(true),
        MultiOption::new("marketing", "Marketing").group("Business teams"),
    ]
}

fn refs(options: &[MultiOption]) -> Vec<&MultiOption> {
    options.iter().collect()
}

#[test]
fn a_query_matches_its_letters_in_order_in_the_value_the_label_and_the_words() {
    let options = teams();
    let find = |q: &str| {
        visible(&options, q)
            .iter()
            .map(|o| o.value.to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(find("").len(), 4, "no query shows all");
    assert_eq!(find("  "), find(""), "blanks count for nothing");
    assert_eq!(find("DSN"), ["design"], "letters in order, in any case");
    assert_eq!(
        find("nsd"),
        Vec::<String>::new(),
        "letters out of order do not match"
    );
    assert_eq!(find("build"), ["engineering"], "a keyword finds the option");
    assert_eq!(find("zzz").len(), 0);
}

#[test]
fn the_active_option_follows_the_cursor_then_the_first_chosen_then_the_first_free() {
    let options = teams();
    let all = refs(&options);
    let none: Vec<SharedString> = Vec::new();
    let value = |s: &str| SharedString::from(s.to_string());
    assert_eq!(active(None, "", &all, &none), Some(&value("design")));
    assert_eq!(
        active(None, "", &all, &[value("marketing")]),
        Some(&value("marketing")),
        "the first chosen value"
    );
    let cursor = (value("engineering"), value(""));
    assert_eq!(
        active(Some(&cursor), "", &all, &[value("marketing")]),
        Some(&value("engineering")),
        "the cursor wins"
    );
    assert_eq!(
        active(Some(&cursor), "e", &all, &none),
        Some(&value("design")),
        "a cursor placed under another query is stale"
    );
    let disabled = (value("product"), value(""));
    assert_eq!(
        active(Some(&disabled), "", &all, &none),
        Some(&value("design")),
        "a disabled option is never active"
    );
    assert_eq!(active(None, "", &[], &none), None);
}

#[test]
fn the_keys_wrap_round_the_options_that_can_be_chosen() {
    let options = teams();
    let all = refs(&options);
    let value = |s: &str| SharedString::from(s.to_string());
    assert_eq!(
        move_active(Some(&value("design")), &all, 1),
        Some(&value("engineering"))
    );
    assert_eq!(
        move_active(Some(&value("engineering")), &all, 1),
        Some(&value("marketing")),
        "skips the disabled option"
    );
    assert_eq!(
        move_active(Some(&value("marketing")), &all, 1),
        Some(&value("design")),
        "wraps at the end"
    );
    assert_eq!(
        move_active(Some(&value("design")), &all, -1),
        Some(&value("marketing")),
        "wraps at the start"
    );
    assert_eq!(move_active(None, &[], 1), None);
}

#[test]
fn the_list_height_counts_rows_groups_and_the_empty_message() {
    let options = teams();
    let all = refs(&options);
    // Two groups: 12 padding, 2 x (4 + 28.32 label), and 4 rows of 36.
    assert!((content_height(&all) - (12. + 2. * (4. + LABEL) + 4. * ROW)).abs() < 0.01);
    assert!((content_height(&[]) - (12. + EMPTY)).abs() < 0.01);
    let plain = [MultiOption::new("a", "A")];
    assert!(
        (content_height(&refs(&plain)) - (12. + 4. + ROW)).abs() < 0.01,
        "no label without a group"
    );
}

fn open<'a>(
    values: &[&str],
    reduce: bool,
    cx: &'a mut TestAppContext,
) -> (Entity<MultiSelect>, &'a mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::motion::clock::freeze();
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
    });
    let values: Vec<SharedString> = values
        .iter()
        .map(|v| SharedString::from(v.to_string()))
        .collect();
    let (select, cx) = cx.add_window_view(move |window, cx| {
        MultiSelect::new("teams", teams(), window, cx).with_values(values)
    });
    cx.simulate_resize(size(px(500.), px(500.)));
    frames(&select, cx, 4);
    (select, cx)
}

fn frames(select: &Entity<MultiSelect>, cx: &mut VisualTestContext, n: usize) {
    for _ in 0..n {
        cx.run_until_parked();
        select.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
}

fn click(name: &'static str, cx: &mut VisualTestContext) {
    let at = cx
        .debug_bounds(name)
        .unwrap_or_else(|| panic!("{name} is not drawn"))
        .center();
    cx.simulate_click(at, gpui_kit::Modifiers::default());
    cx.run_until_parked();
}

fn values(select: &Entity<MultiSelect>, cx: &mut VisualTestContext) -> Vec<String> {
    select.read_with(cx, |s, _| {
        s.values().iter().map(|v| v.to_string()).collect()
    })
}

#[gpui_kit::test]
fn a_press_on_the_field_opens_the_list_and_a_row_toggles_its_option_and_keeps_the_list_open(
    cx: &mut TestAppContext,
) {
    let (select, cx) = open(&["design"], true, cx);
    assert!(!select.read_with(cx, |s, _| s.is_open()));
    assert!(
        cx.debug_bounds("multi-option-design").is_none(),
        "closed: no rows"
    );
    click("multi-field", cx);
    frames(&select, cx, 4);
    assert!(select.read_with(cx, |s, _| s.is_open()));
    click("multi-option-marketing", cx);
    frames(&select, cx, 3);
    assert_eq!(values(&select, cx), ["design", "marketing"]);
    assert!(
        select.read_with(cx, |s, _| s.is_open()),
        "the list stays open"
    );
    click("multi-option-design", cx);
    assert_eq!(values(&select, cx), ["marketing"], "a chosen row lets go");
    click("multi-option-product", cx);
    assert_eq!(
        values(&select, cx),
        ["marketing"],
        "a disabled row does nothing"
    );
}

#[gpui_kit::test]
fn the_cross_on_a_chip_and_backspace_on_an_empty_field_take_a_chip_away(cx: &mut TestAppContext) {
    let (select, cx) = open(&["design", "engineering", "marketing"], true, cx);
    click("multi-remove-engineering", cx);
    assert_eq!(values(&select, cx), ["design", "marketing"]);
    assert!(
        !select.read_with(cx, |s, _| s.is_open()),
        "the cross does not open the list"
    );
    click("multi-field", cx);
    frames(&select, cx, 3);
    cx.simulate_keystrokes("backspace");
    assert_eq!(values(&select, cx), ["design"], "the last chip goes first");
    cx.simulate_keystrokes("backspace");
    cx.simulate_keystrokes("backspace");
    assert!(values(&select, cx).is_empty(), "nothing left to take");
}

#[gpui_kit::test]
fn the_arrow_keys_move_the_active_row_enter_chooses_it_and_escape_closes(cx: &mut TestAppContext) {
    let (select, cx) = open(&[], true, cx);
    click("multi-field", cx);
    frames(&select, cx, 4);
    cx.simulate_keystrokes("down");
    cx.simulate_keystrokes("enter");
    assert_eq!(
        values(&select, cx),
        ["engineering"],
        "the first row is active, so down goes to the second"
    );
    cx.simulate_keystrokes("up");
    cx.simulate_keystrokes("enter");
    assert_eq!(values(&select, cx), ["engineering", "design"]);
    cx.simulate_keystrokes("escape");
    frames(&select, cx, 3);
    assert!(!select.read_with(cx, |s, _| s.is_open()));
    assert!(select.read_with(cx, |s, _| s.query().is_empty()));
}

#[gpui_kit::test]
fn a_press_in_the_hole_reaches_the_field_and_a_press_elsewhere_closes_the_list(
    cx: &mut TestAppContext,
) {
    let (select, cx) = open(&[], true, cx);
    click("multi-field", cx);
    frames(&select, cx, 4);
    assert!(select.read_with(cx, |s, _| s.is_open()));
    click("multi-field", cx);
    assert!(
        select.read_with(cx, |s, _| s.is_open()),
        "a press on the field does not close the list"
    );
    cx.simulate_click(
        gpui_kit::point(px(490.), px(490.)),
        gpui_kit::Modifiers::default(),
    );
    cx.run_until_parked();
    assert!(
        !select.read_with(cx, |s, _| s.is_open()),
        "a press elsewhere closes it"
    );
}

#[gpui_kit::test]
fn with_reduce_motion_a_chip_that_goes_leaves_no_wipe_and_the_list_is_at_full_height(
    cx: &mut TestAppContext,
) {
    let (select, cx) = open(&["design", "engineering"], true, cx);
    click("multi-remove-design", cx);
    assert!(
        select.read_with(cx, |s, _| s.leaving.is_empty()),
        "no ghost chip"
    );
    click("multi-field", cx);
    frames(&select, cx, 3);
    let (height, want) = select.read_with(cx, |s, _| (s.height.value(), s.height.target()));
    assert_eq!(height, want, "the list jumps to its height");
    assert!(height > 100., "and it has rows: {height}");
}

#[gpui_kit::test]
fn with_motion_a_chip_that_goes_leaves_a_wipe_that_runs_out(cx: &mut TestAppContext) {
    let (select, cx) = open(&["design", "engineering"], false, cx);
    click("multi-remove-design", cx);
    assert_eq!(
        select.read_with(cx, |s, _| s.leaving.len()),
        1,
        "a ghost chip wipes away"
    );
    for _ in 0..60 {
        crate::motion::clock::advance(std::time::Duration::from_millis(16));
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(16));
        frames(&select, cx, 1);
    }
    assert!(
        select.read_with(cx, |s, _| s.leaving.is_empty()),
        "the wipe ended"
    );
}

#[gpui_kit::test]
fn a_chip_is_as_wide_as_its_label_and_chips_sit_side_by_side(cx: &mut TestAppContext) {
    let (select, cx) = open(&["design", "engineering"], true, cx);
    frames(&select, cx, 3);
    let (a, b) = (
        cx.debug_bounds("multi-chip-design").unwrap(),
        cx.debug_bounds("multi-chip-engineering").unwrap(),
    );
    assert!(
        f32::from(a.size.width) > 60.,
        "Design + cross is wider than 60px: {a:?}"
    );
    assert!(
        f32::from(b.size.width) > f32::from(a.size.width),
        "the longer label makes the wider chip"
    );
    assert!(b.left() >= a.right(), "no overlap: {a:?} {b:?}");
    assert_eq!(f32::from(a.size.height), CHIP_HEIGHT);
}
