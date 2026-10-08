use super::*;
use gpui_kit::ParentElement;
use gpui_kit::Styled;

fn hunk(id: &str, removed: Range<usize>, added: Range<usize>) -> InlineHunk {
    InlineHunk::new(id, removed, added)
}

#[test]
fn a_hunk_holds_both_sides_and_its_old_rows_come_first() {
    let h = hunk("a", 4..6, 6..9);
    assert_eq!(h.rows(), 4..9);
    assert_eq!(h.surviving(Decision::Accept), 6..9);
    assert_eq!(h.closing(Decision::Accept), 4..6);
    assert_eq!(h.surviving(Decision::Reject), 4..6);
    assert_eq!(h.closing(Decision::Reject), 6..9);
}

#[test]
fn a_pure_insertion_has_nothing_to_delete_on_accept() {
    let h = hunk("a", 4..4, 4..7);
    assert!(h.closing(Decision::Accept).is_empty());
    assert_eq!(h.closing(Decision::Reject), 4..7);
    assert_eq!(
        plan_edits(&[(h, Decision::Accept)]),
        Vec::<Range<usize>>::new()
    );
}

#[test]
fn a_pure_deletion_has_nothing_to_delete_on_reject() {
    let h = hunk("a", 2..5, 5..5);
    assert_eq!(h.closing(Decision::Accept), 2..5);
    assert!(h.closing(Decision::Reject).is_empty());
}

/// Applying an edit shifts every row below it, so the edits must run from the bottom of the file up.
/// Running them top down would delete the wrong rows for every hunk after the first.
#[test]
fn edits_run_from_the_bottom_of_the_file_upwards() {
    let plan = plan_edits(&[
        (hunk("a", 2..4, 4..5), Decision::Accept),
        (hunk("b", 20..21, 21..23), Decision::Accept),
        (hunk("c", 10..12, 12..13), Decision::Reject),
    ]);
    assert_eq!(plan, vec![20..21, 12..13, 2..4]);
}

#[test]
fn a_caret_below_a_deleted_block_moves_up_by_its_height() {
    assert_eq!(hold_caret(9, &[2..4, 7..8]), 6);
}

#[test]
fn a_caret_inside_a_deleted_block_lands_where_the_block_started() {
    assert_eq!(hold_caret(3, std::slice::from_ref(&(2..4))), 2);
    assert_eq!(hold_caret(9, &[2..4, 8..10]), 6);
}

#[test]
fn a_caret_above_every_deleted_block_does_not_move() {
    assert_eq!(hold_caret(1, &[2..4, 7..8]), 1);
}

#[test]
fn an_empty_closing_range_never_moves_the_caret() {
    assert_eq!(hold_caret(5, std::slice::from_ref(&(3..3))), 5);
}

#[test]
fn a_row_becomes_the_byte_offset_it_starts_at() {
    let text = "one\ntwo\nthree\n";
    assert_eq!(row_to_byte(text, 0), 0);
    assert_eq!(row_to_byte(text, 1), 4);
    assert_eq!(row_to_byte(text, 2), 8);
    assert_eq!(rows_to_bytes(text, &(1..3)), 4..14);
}

/// A hunk at the end of a file with no trailing newline still needs a range to delete, so the row one
/// past the last resolves to the length of the text rather than to nothing.
#[test]
fn a_row_past_the_last_line_is_the_end_of_the_text() {
    let text = "one\ntwo";
    assert_eq!(row_to_byte(text, 2), 7);
    assert_eq!(row_to_byte(text, 40), 7);
    assert_eq!(rows_to_bytes(text, &(1..2)), 4..7);
}

/// The offsets are bytes, not characters. A range measured in characters would cut a multi-byte
/// character in half and corrupt the file.
#[test]
fn row_offsets_are_measured_in_bytes_not_characters() {
    let text = "café\nnaïve\n";
    assert_eq!(row_to_byte(text, 1), 6);
    assert_eq!(row_to_byte(text, 2), 13);
}

#[test]
fn each_side_of_each_hunk_gets_a_wash_and_an_empty_side_none() {
    let hunks = [hunk("a", 2..4, 4..5), hunk("insert", 9..9, 9..11)];
    assert_eq!(
        washes(&hunks),
        vec![(2..4, false), (4..5, true), (9..11, true)]
    );
}

#[test]
fn a_decided_hunk_stops_counting_as_pending() {
    let hunks = [hunk("a", 1..2, 2..3), hunk("b", 5..6, 6..7)];
    assert_eq!(pending_count(&hunks, &[]), 2);
    assert_eq!(pending_count(&hunks, &["a".into()]), 1);
    assert_eq!(pending_count(&hunks, &["a".into(), "b".into()]), 0);
}

/// Deleting rows moves everything below them up. A hunk below the deletion whose rows are not moved
/// with it would paint its band, and place its Accept, on the wrong code.
#[test]
fn a_hunk_below_a_resolved_one_slides_up_by_its_height() {
    let hunks = [hunk("a", 1..2, 2..4), hunk("b", 8..9, 9..10)];
    let left = shift_after(&hunks, &"a".into(), &(1..2));
    assert_eq!(left, vec![hunk("b", 7..8, 8..9)]);
}

#[test]
fn a_hunk_above_a_resolved_one_does_not_move() {
    let hunks = [hunk("a", 1..2, 2..4), hunk("b", 8..9, 9..10)];
    let left = shift_after(&hunks, &"b".into(), &(8..9));
    assert_eq!(left, vec![hunk("a", 1..2, 2..4)]);
}

/// A pure insertion accepted deletes nothing, so no other hunk may move.
#[test]
fn an_empty_deletion_moves_no_other_hunk() {
    let hunks = [hunk("a", 1..1, 1..3), hunk("b", 8..9, 9..10)];
    let left = shift_after(&hunks, &"a".into(), &(1..1));
    assert_eq!(left, vec![hunk("b", 8..9, 9..10)]);
}

#[test]
fn the_decided_hunk_is_dropped_from_what_is_left() {
    let hunks = [hunk("a", 1..2, 2..4), hunk("b", 8..9, 9..10)];
    assert_eq!(shift_after(&hunks, &"a".into(), &(1..2)).len(), 1);
    assert_eq!(shift_after(&hunks, &"b".into(), &(8..9)).len(), 1);
}

/// What [`apply`] does to the text, run on a `String` so it needs no window.
///
/// This mirrors `apply` exactly: every byte range is measured against the ORIGINAL text, and the
/// ranges are applied in the order [`plan_edits`] returns them. If that order were wrong, the second
/// edit would cut the wrong bytes and this would show it.
fn apply_to_string(text: &str, plan: &[Range<usize>]) -> String {
    let mut out = text.to_string();
    for rows in plan {
        out.replace_range(rows_to_bytes(text, rows), "");
    }
    out
}

const FIXTURE: &str = "pub struct Config {\n    pub width: u32,\n    pub width: u32,\n    pub height: u32,\n}\n\nimpl Config {\n    pub fn new() -> Self {\n        Self { width: 80 }\n        Self { width: 80, height: 24 }\n    }\n}\n";

fn fixture_hunks() -> Vec<InlineHunk> {
    vec![hunk("field", 1..2, 2..4), hunk("ctor", 8..9, 9..10)]
}

#[test]
fn accepting_both_hunks_leaves_the_agents_file() {
    let hunks = fixture_hunks();
    let plan = plan_edits(&[
        (hunks[0].clone(), Decision::Accept),
        (hunks[1].clone(), Decision::Accept),
    ]);
    let out = apply_to_string(FIXTURE, &plan);
    assert_eq!(
        out,
        "pub struct Config {\n    pub width: u32,\n    pub height: u32,\n}\n\nimpl Config {\n    pub fn new() -> Self {\n        Self { width: 80, height: 24 }\n    }\n}\n"
    );
}

#[test]
fn rejecting_both_hunks_leaves_the_file_as_it_was() {
    let hunks = fixture_hunks();
    let plan = plan_edits(&[
        (hunks[0].clone(), Decision::Reject),
        (hunks[1].clone(), Decision::Reject),
    ]);
    let out = apply_to_string(FIXTURE, &plan);
    assert_eq!(
        out,
        "pub struct Config {\n    pub width: u32,\n}\n\nimpl Config {\n    pub fn new() -> Self {\n        Self { width: 80 }\n    }\n}\n"
    );
}

/// One hunk at a time is the normal case, and the second hunk's rows must have moved with it.
#[test]
fn accepting_one_hunk_then_the_other_matches_accepting_both() {
    let hunks = fixture_hunks();
    let first = plan_edits(&[(hunks[0].clone(), Decision::Accept)]);
    let after_first = apply_to_string(FIXTURE, &first);
    let left = shift_after(&hunks, &"field".into(), &first[0]);
    assert_eq!(
        left,
        vec![hunk("ctor", 7..8, 8..9)],
        "the second hunk slid up by the row that went"
    );

    let second = plan_edits(&[(left[0].clone(), Decision::Accept)]);
    let out = apply_to_string(&after_first, &second);
    let both = apply_to_string(
        FIXTURE,
        &plan_edits(&[
            (hunks[0].clone(), Decision::Accept),
            (hunks[1].clone(), Decision::Accept),
        ]),
    );
    assert_eq!(
        out, both,
        "two decisions in turn must land where one pass lands"
    );
}

/// Mixing the decisions must keep the old field and take the agent's constructor.
#[test]
fn a_reject_and_an_accept_each_keep_their_own_side() {
    let hunks = fixture_hunks();
    let plan = plan_edits(&[
        (hunks[0].clone(), Decision::Reject),
        (hunks[1].clone(), Decision::Accept),
    ]);
    let out = apply_to_string(FIXTURE, &plan);
    assert!(
        !out.contains("pub height"),
        "the rejected field must be gone: {out}"
    );
    assert!(
        out.contains("height: 24"),
        "the accepted constructor must stay: {out}"
    );
}

/// The fixture text: rows 1 and 2 are a hunk's old and new rows, and rows 4 to 6 another's.
const TRACKED: &str = "a\nold\nnew\nb\nold2\nnew2\nnew3\nc";

fn tracked() -> Vec<InlineHunk> {
    vec![hunk("one", 1..2, 2..3), hunk("two", 4..5, 5..7)]
}

#[test]
fn an_edit_below_every_hunk_moves_nothing() {
    let after = TRACKED.replace("\nc", "\nc\nd");
    assert_eq!(track_edit(&tracked(), TRACKED, &after), tracked());
}

#[test]
fn a_line_added_above_moves_every_hunk_down() {
    let after = format!("top\n{TRACKED}");
    assert_eq!(
        track_edit(&tracked(), TRACKED, &after),
        vec![hunk("one", 2..3, 3..4), hunk("two", 5..6, 6..8)]
    );
}

#[test]
fn a_line_added_right_at_a_hunks_first_row_pushes_it_down() {
    let after = TRACKED.replacen("a\nold", "a\nextra\nold", 1);
    assert_eq!(
        track_edit(&tracked(), TRACKED, &after),
        vec![hunk("one", 2..3, 3..4), hunk("two", 5..6, 6..8)]
    );
}

#[test]
fn a_line_split_inside_a_hunk_grows_that_side() {
    let after = TRACKED.replacen("\nnew\n", "\nne\nw\n", 1);
    assert_eq!(
        track_edit(&tracked(), TRACKED, &after),
        vec![hunk("one", 1..2, 2..4), hunk("two", 5..6, 6..8)]
    );
}

#[test]
fn a_line_added_between_old_and_new_joins_the_agents_side() {
    let after = TRACKED.replacen("old\nnew", "old\nmine\nnew", 1);
    assert_eq!(
        track_edit(&tracked(), TRACKED, &after),
        vec![hunk("one", 1..2, 2..4), hunk("two", 5..6, 6..8)]
    );
}

#[test]
fn a_line_deleted_inside_a_hunk_shrinks_that_side() {
    let after = TRACKED.replacen("new2\nnew3", "new2", 1);
    assert_eq!(
        track_edit(&tracked(), TRACKED, &after),
        vec![hunk("one", 1..2, 2..3), hunk("two", 4..5, 5..6)]
    );
}

#[test]
fn a_hunk_whose_rows_were_all_deleted_is_dropped() {
    let after = TRACKED.replacen("\nold\nnew", "", 1);
    assert_eq!(
        track_edit(&tracked(), TRACKED, &after),
        vec![hunk("two", 2..3, 3..5)]
    );
}

#[test]
fn typing_inside_a_row_moves_nothing() {
    let after = TRACKED.replacen("new2", "new2 and more", 1);
    assert_eq!(track_edit(&tracked(), TRACKED, &after), tracked());
}

#[test]
fn the_keyboard_decides_the_hunk_under_the_caret() {
    let hunks = tracked();
    assert_eq!(
        hunk_at_row(&hunks, 1).map(|h| h.id.as_ref()),
        Some("one"),
        "an old row"
    );
    assert_eq!(
        hunk_at_row(&hunks, 6).map(|h| h.id.as_ref()),
        Some("two"),
        "the last new row"
    );
    assert_eq!(
        hunk_at_row(&hunks, 3),
        None,
        "a row between hunks belongs to neither"
    );
    assert_eq!(hunk_at_row(&hunks, 7), None);
}

#[test]
fn undo_and_redo_of_a_decision_bring_its_hunks_back() {
    let mut history = DecisionHistory::default();
    let before = ("old\nnew".to_string(), vec![hunk("one", 0..1, 1..2)]);
    let after = ("new".to_string(), vec![]);
    history.record(before.clone(), after.clone());
    assert_eq!(
        history.hunks_for("old\nnew"),
        Some(before.1),
        "undo: the hunk is back"
    );
    assert_eq!(
        history.hunks_for("new"),
        Some(after.1),
        "redo: it is decided again"
    );
    assert_eq!(
        history.hunks_for("something else"),
        None,
        "any other text is an ordinary edit"
    );
}

#[test]
fn the_newest_decision_wins_when_two_share_a_text() {
    let mut history = DecisionHistory::default();
    history.record(
        ("a".into(), vec![hunk("x", 0..1, 1..1)]),
        ("b".into(), vec![]),
    );
    history.record(
        ("b".into(), vec![hunk("y", 0..1, 1..1)]),
        ("c".into(), vec![]),
    );
    assert_eq!(
        history.hunks_for("b").map(|h| h[0].id.clone()),
        Some("y".into())
    );
}

#[test]
fn the_bar_is_short_below_the_width_where_it_would_cover_the_code() {
    assert!(compact_bar(480.) && compact_bar(COMPACT_BELOW - 1.));
    assert!(
        !compact_bar(COMPACT_BELOW) && !compact_bar(1100.) && !compact_bar(f32::MAX),
        "the first frame draws the full bar"
    );
}

mod narrow {
    use super::*;
    use gpui_kit::{Context, Render, TestAppContext, Window, div, px};

    struct Card {
        state: gpui_kit::Entity<gpui_kit::component::input::EditorState>,
        width: f32,
    }
    impl Render for Card {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui_kit::IntoElement {
            let hunks = vec![hunk("h", 1..2, 2..3)];
            div().w(px(self.width)).child(
                InlineReview::new("r", &self.state, hunks)
                    .current("h")
                    .height(px(200.))
                    .on_decide(|_, _, _, _| {}),
            )
        }
    }

    fn bar_width(card_width: f32, cx: &mut TestAppContext) -> f32 {
        cx.update(|cx| {
            gpui_kit::init(cx);
            crate::theme::set_appearance(crate::theme::Appearance::Light, cx);
        });
        let (_, cx) = cx.add_window_view(move |window, cx| Card {
            state: crate::code_editor::CodeEditor::state(
                "x.rs",
                "one\ntwo old\ntwo new\nthree\n",
                window,
                cx,
            ),
            width: card_width,
        });
        for _ in 0..4 {
            cx.run_until_parked();
            cx.update(|window, _| window.refresh());
        }
        f32::from(
            cx.debug_bounds("hunk-bar")
                .expect("the bar is drawn")
                .size
                .width,
        )
    }

    #[gpui_kit::test]
    fn on_a_card_of_480_px_the_bar_keeps_two_icons_and_a_wide_card_keeps_its_words(
        cx: &mut TestAppContext,
    ) {
        let narrow = bar_width(480., cx);
        assert!(narrow < 90., "the short bar is {narrow}px wide");
    }

    #[gpui_kit::test]
    fn a_wide_card_keeps_the_words(cx: &mut TestAppContext) {
        let wide = bar_width(900., cx);
        assert!(wide > 150., "the full bar is {wide}px wide");
    }
}
