use super::host::{Options, shown_with_log};
use gpui_kit::{KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, TestAppContext, VisualTestContext};

fn press(cx: &mut VisualTestContext, name: &'static str) {
    let at = cx.debug_bounds(name).unwrap_or_else(|| panic!("no {name}")).center();
    cx.simulate_click(at, Modifiers::default());
    cx.run_until_parked();
}

/// Enter or Space, down and up, as the keyboard sends it.
fn key(cx: &mut VisualTestContext, name: &str) {
    let keystroke = Keystroke::parse(name).unwrap();
    cx.simulate_event(KeyDownEvent { keystroke: keystroke.clone(), is_held: false, prefer_character_input: false });
    cx.simulate_event(KeyUpEvent { keystroke });
    cx.run_until_parked();
}

#[gpui_kit::test]
fn a_press_on_a_range_a_tile_a_group_or_a_session_tells_the_app(cx: &mut TestAppContext) {
    let (cx, log) = shown_with_log(Options::default(), cx);
    press(cx, "usage-range-7");
    press(cx, "usage-tile-source-codex");
    press(cx, "usage-tile-all");
    press(cx, "usage-group-Claude Code · 2 accounts");
    press(cx, "usage-session-s1");
    assert_eq!(
        *log.borrow(),
        vec![
            "range 7".to_string(),
            "select Source(\"codex\")".into(),
            "select All".into(),
            "select Group(\"Claude Code · 2 accounts\")".into(),
            "expand Some(\"s1\")".into(),
        ]
    );
}

#[gpui_kit::test]
fn a_press_on_the_open_session_asks_to_close_it_and_its_days_show(cx: &mut TestAppContext) {
    let (cx, log) = shown_with_log(Options { expanded: Some("s1"), ..Options::default() }, cx);
    assert!(cx.debug_bounds("usage-session-detail").is_some(), "the open session shows its days");
    press(cx, "usage-session-s1");
    assert_eq!(*log.borrow(), vec!["expand None".to_string()]);
}

#[gpui_kit::test]
fn a_closed_session_shows_no_days(cx: &mut TestAppContext) {
    let (cx, _) = shown_with_log(Options::default(), cx);
    assert!(cx.debug_bounds("usage-session-detail").is_none());
}

#[gpui_kit::test]
fn a_focused_tile_or_row_is_pressed_by_enter_and_space(cx: &mut TestAppContext) {
    let (cx, log) = shown_with_log(Options::default(), cx);
    press(cx, "usage-tile-source-codex");
    key(cx, "enter");
    key(cx, "space");
    assert_eq!(
        *log.borrow(),
        vec!["select Source(\"codex\")".to_string(); 3],
        "the press took focus, then Enter and Space pressed it again"
    );
    press(cx, "usage-session-s1");
    key(cx, "enter");
    assert_eq!(log.borrow().len(), 5);
    assert_eq!(log.borrow()[4], "expand Some(\"s1\")");
}

#[gpui_kit::test]
fn an_empty_selection_shows_its_words_in_place_of_the_chart(cx: &mut TestAppContext) {
    let (cx, _) = shown_with_log(Options { empty: Some("No use in this range."), ..Options::default() }, cx);
    assert!(cx.debug_bounds("usage-empty").is_some());
    assert!(cx.debug_bounds("usage-day-0").is_none());
    assert!(cx.debug_bounds("usage-session-s1").is_none());
    assert!(cx.debug_bounds("usage-tile-all").is_some(), "the tiles stay, to change the selection");
}
