use super::*;

const DIFF: &str = "\
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,3 +10,4 @@ fn main() {
     let a = 1;
-    let b = 2;
+    let b = 3;
+    let c = 4;
     run(a, b);
";

#[test]
fn file_headers_are_dropped_and_the_hunk_is_kept() {
    let lines = DiffLine::parse(DIFF);
    assert_eq!(lines[0].kind, DiffLineKind::Hunk);
    assert_eq!(lines.len(), 6);
}

#[test]
fn lines_are_numbered_from_the_hunk_start() {
    let lines = DiffLine::parse(DIFF);
    let numbers: Vec<_> = lines.iter().map(|l| (l.kind, l.new_line, l.old_line)).collect();
    assert_eq!(
        numbers,
        vec![
            (DiffLineKind::Hunk, None, None),
            (DiffLineKind::Context, Some(10), Some(10)),
            (DiffLineKind::Removed, None, Some(11)),
            (DiffLineKind::Added, Some(11), None),
            (DiffLineKind::Added, Some(12), None),
            (DiffLineKind::Context, Some(13), Some(12)),
        ]
    );
}

#[test]
fn the_sign_is_not_part_of_the_text() {
    let lines = DiffLine::parse(DIFF);
    assert_eq!(lines[2].text.as_ref(), "    let b = 2;");
}

#[test]
fn stats_count_added_and_removed_lines() {
    assert_eq!(diff_stats(&DiffLine::parse(DIFF)), (2, 1));
}

#[test]
fn a_removed_line_that_starts_with_two_dashes_is_kept() {
    let lines = DiffLine::parse("@@ -1,2 +1,1 @@\n--- SQL comment\n keep\n");
    assert_eq!(lines[1].kind, DiffLineKind::Removed);
    assert_eq!(lines[1].text.as_ref(), "-- SQL comment");
    assert_eq!(diff_stats(&lines), (0, 1));
}

#[test]
fn an_added_line_that_starts_with_two_pluses_is_kept() {
    let lines = DiffLine::parse("@@ -1,0 +1,1 @@\n+++counter;\n");
    assert_eq!(lines[1].kind, DiffLineKind::Added);
}

#[test]
fn the_git_preamble_is_dropped() {
    let lines = DiffLine::parse("diff --git a/x b/x\nindex 1..2 100644\n--- a/x\n+++ b/x\n@@ -1,1 +1,1 @@\n-a\n+b\n");
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].kind, DiffLineKind::Hunk);
}

#[test]
fn no_newline_markers_are_skipped_and_do_not_count() {
    let lines = DiffLine::parse("@@ -1,1 +1,1 @@\n-a\n\\ No newline at end of file\n+b\n c\n");
    let kinds: Vec<_> = lines.iter().map(|l| (l.kind, l.new_line, l.old_line)).collect();
    assert_eq!(
        kinds,
        vec![
            (DiffLineKind::Hunk, None, None),
            (DiffLineKind::Removed, None, Some(1)),
            (DiffLineKind::Added, Some(1), None),
            (DiffLineKind::Context, Some(2), Some(2)),
        ]
    );
}

const TWO_FILE_DIFF: &str = "\
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,3 +10,4 @@ fn main() {
     let a = 1;
-    let b = 2;
+    let b = 3;
     run(a, b);
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,2 +1,2 @@
-old
+new
";

#[test]
fn a_second_files_diff_header_resets_the_hunk_state() {
    let lines = DiffLine::parse(TWO_FILE_DIFF);
    let kinds: Vec<_> = lines.iter().map(|l| l.kind).collect();
    assert_eq!(
        kinds,
        vec![
            DiffLineKind::Hunk,
            DiffLineKind::Context,
            DiffLineKind::Removed,
            DiffLineKind::Added,
            DiffLineKind::Context,
            DiffLineKind::Hunk,
            DiffLineKind::Removed,
            DiffLineKind::Added,
        ]
    );
    // Without the reset, the second file's own `---`/`+++` headers are misread as content lines.
    assert_eq!(lines[6].text.as_ref(), "old");
    assert_eq!(lines[7].text.as_ref(), "new");
}

mod tail {
    use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, UniformListScrollHandle, Window, div, px, size};

    use super::super::*;
    use crate::theme::{Appearance, set_appearance};

    struct Host {
        rows: usize,
        status: FileDiffStatus,
        scroll: UniformListScrollHandle,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let lines = (0..self.rows)
                .map(|n| DiffLine { kind: DiffLineKind::Added, old_line: None, new_line: Some(n as u32 + 1), text: format!("line {n}").into() })
                .collect();
            div().w(px(600.)).child(FileDiff::new("tail", "src/main.rs", lines).status(self.status).scroll_handle(self.scroll.clone()))
        }
    }

    fn settle(host: &gpui_kit::Entity<Host>, cx: &mut gpui_kit::VisualTestContext) {
        for _ in 0..4 {
            cx.run_until_parked();
            host.update(cx, |_, cx| cx.notify());
        }
        cx.run_until_parked();
    }

    fn open(rows: usize, status: FileDiffStatus, cx: &mut TestAppContext) -> (gpui_kit::Entity<Host>, &mut gpui_kit::VisualTestContext, UniformListScrollHandle) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Light, cx);
            cx.set_reduce_motion(true);
        });
        let scroll = UniformListScrollHandle::new();
        let handle = scroll.clone();
        let (host, cx) = cx.add_window_view(move |_, _| Host { rows, status, scroll });
        cx.simulate_resize(size(px(700.), px(700.)));
        settle(&host, cx);
        (host, cx, handle)
    }

    /// How far the rows are from their end, in pixels: 0 at the end.
    fn from_end(handle: &UniformListScrollHandle) -> f32 {
        let base = handle.0.borrow().base_handle.clone();
        f32::from(base.max_offset().y + base.offset().y)
    }

    #[gpui_kit::test]
    fn a_streaming_diff_follows_its_newest_row(cx: &mut TestAppContext) {
        let (host, cx, handle) = open(30, FileDiffStatus::Streaming, cx);
        assert!(f32::from(handle.0.borrow().base_handle.max_offset().y) > 0., "30 rows are more than the viewport shows");
        assert!(from_end(&handle).abs() < 1., "the end is in view: {}", from_end(&handle));
        host.update(cx, |h, cx| {
            h.rows = 60;
            cx.notify();
        });
        settle(&host, cx);
        assert!(from_end(&handle).abs() < 1., "it follows the new rows: {}", from_end(&handle));
    }

    #[gpui_kit::test]
    fn a_complete_diff_stays_where_the_reader_left_it(cx: &mut TestAppContext) {
        let (host, cx, handle) = open(30, FileDiffStatus::Complete, cx);
        assert!(from_end(&handle) > 100., "a finished diff starts at its top: {}", from_end(&handle));
        host.update(cx, |h, cx| {
            h.rows = 60;
            cx.notify();
        });
        settle(&host, cx);
        assert!(from_end(&handle) > 100.);
    }
}

#[test]
fn a_diff_parts_from_the_panel_and_from_a_box_it_sits_in() {
    use crate::theme::{contrast, mix};
    for theme in crate::themes::all() {
        let shown = |fill: gpui_kit::Hsla, on: gpui_kit::Hsla| {
            mix(on, gpui_kit::Hsla { a: 1., ..fill }, fill.a)
        };
        let boxed = super::helpers::fill(theme, false);
        assert!(
            contrast(shown(boxed, theme.card), theme.card) >= 1.05,
            "{}: a box on the panel",
            theme.name
        );
        let inset = super::helpers::fill(theme, true);
        let well = shown(inset, theme.card_strong);
        assert!(
            contrast(well, theme.card_strong) >= 1.05,
            "{}: a well in a box: {:.3}",
            theme.name,
            contrast(well, theme.card_strong)
        );
    }
}

mod clipped {
    use std::{cell::Cell, rc::Rc};

    use gpui_kit::{Context, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};

    use super::super::*;
    use crate::theme::{Appearance, set_appearance};

    struct Host {
        rows: usize,
        status: FileDiffStatus,
        opened: Rc<Cell<usize>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let lines = (0..self.rows)
                .map(|n| DiffLine { kind: DiffLineKind::Added, old_line: None, new_line: Some(n as u32 + 1), text: format!("line {n}").into() })
                .collect();
            let opened = self.opened.clone();
            div().w(px(600.)).child(
                FileDiff::new("clipped", "src/main.rs", lines)
                    .status(self.status)
                    .preview_rows(8)
                    .on_open(move |_, _| opened.set(opened.get() + 1)),
            )
        }
    }

    fn settle(host: &gpui_kit::Entity<Host>, cx: &mut gpui_kit::VisualTestContext) {
        for _ in 0..4 {
            cx.run_until_parked();
            host.update(cx, |_, cx| cx.notify());
        }
        cx.run_until_parked();
    }

    fn open(rows: usize, status: FileDiffStatus, cx: &mut TestAppContext) -> (gpui_kit::Entity<Host>, &mut gpui_kit::VisualTestContext, Rc<Cell<usize>>) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Light, cx);
            cx.set_reduce_motion(true);
        });
        let opened = Rc::new(Cell::new(0));
        let seen = opened.clone();
        let (host, cx) = cx.add_window_view(move |_, _| Host { rows, status, opened });
        cx.simulate_resize(size(px(700.), px(900.)));
        settle(&host, cx);
        (host, cx, seen)
    }

    fn height(cx: &mut gpui_kit::VisualTestContext, selector: &'static str) -> f32 {
        f32::from(cx.debug_bounds(selector).unwrap().size.height)
    }

    fn press_body(cx: &mut gpui_kit::VisualTestContext) {
        let at = cx.debug_bounds("diff-rows").map(|b| b.center()).or_else(|| cx.debug_bounds("diff-body").map(|b| b.origin + gpui_kit::point(px(20.), px(20.)))).unwrap();
        cx.simulate_click(at, Modifiers::none());
    }

    #[gpui_kit::test]
    fn a_long_diff_shows_only_its_preview_rows_and_does_not_scroll(cx: &mut TestAppContext) {
        let (_, cx, opened) = open(30, FileDiffStatus::Complete, cx);
        assert_eq!(height(cx, "diff-rows"), 8. * ROW_HEIGHT);
        assert_eq!(opened.get(), 0);
    }

    #[gpui_kit::test]
    fn a_short_diff_is_as_tall_as_its_rows(cx: &mut TestAppContext) {
        let (_, cx, _) = open(3, FileDiffStatus::Complete, cx);
        assert_eq!(height(cx, "diff-rows"), 3. * ROW_HEIGHT);
    }

    #[gpui_kit::test]
    fn pressing_it_opens_it_and_tells_the_owner_once(cx: &mut TestAppContext) {
        let (host, cx, opened) = open(30, FileDiffStatus::Complete, cx);
        let before = height(cx, "diff-body");
        press_body(cx);
        settle(&host, cx);
        assert_eq!(opened.get(), 1);
        assert!(cx.debug_bounds("diff-rows").is_none(), "the clipped rows give way to the taller view");
        assert!(height(cx, "diff-body") > before + 100., "it grew: {} from {before}", height(cx, "diff-body"));
    }

    #[gpui_kit::test]
    fn pressing_it_again_folds_it_without_telling_the_owner(cx: &mut TestAppContext) {
        let (host, cx, opened) = open(30, FileDiffStatus::Complete, cx);
        press_body(cx);
        settle(&host, cx);
        press_body(cx);
        settle(&host, cx);
        assert_eq!(opened.get(), 1);
        assert_eq!(height(cx, "diff-rows"), 8. * ROW_HEIGHT);
    }
}

mod capped {
    use gpui_kit::{Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};

    use super::super::*;
    use crate::theme::{Appearance, set_appearance};

    struct Host(usize);

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let lines = (0..self.0)
                .map(|n| DiffLine { kind: DiffLineKind::Added, old_line: None, new_line: Some(n as u32 + 1), text: format!("line {n}").into() })
                .collect();
            let diff = FileDiff::new("capped", "src/main.rs", lines).status(FileDiffStatus::Complete).collapse_on_complete(false).max_height(300.);
            div().size_full().flex().flex_col().child(div().debug_selector(|| "capped".into()).child(diff))
        }
    }

    fn height(rows: usize, cx: &mut TestAppContext) -> f32 {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Light, cx);
            cx.set_reduce_motion(true);
        });
        let (host, cx) = cx.add_window_view(move |_, _| Host(rows));
        cx.simulate_resize(size(px(700.), px(900.)));
        for _ in 0..4 {
            cx.run_until_parked();
            host.update(cx, |_, cx| cx.notify());
        }
        f32::from(cx.debug_bounds("capped").unwrap().size.height)
    }

    /// A long file's rows scroll in the height they are given, and only the rows in sight are laid out.
    #[gpui_kit::test]
    fn a_long_diff_scrolls_in_its_max_height(cx: &mut TestAppContext) {
        let h = height(500, cx);
        assert!(h <= 300. + crate::tool_call::CARD_HEADER_HEIGHT + 8., "500 rows in 300 px: {h}");
    }

    #[gpui_kit::test]
    fn a_short_diff_under_its_max_height_is_as_tall_as_its_rows(cx: &mut TestAppContext) {
        let h = height(3, cx);
        assert!(h < 3. * ROW_HEIGHT + crate::tool_call::CARD_HEADER_HEIGHT + 8., "3 rows: {h}");
    }
}

/// The wheel over a diff in a panel: a clipped diff does not scroll, so the panel takes the wheel; an opened one scrolls its
/// own rows first; and one folded back is a clipped diff again.
mod wheel {
    use gpui_kit::{
        Context, InteractiveElement, IntoElement, Modifiers, ParentElement, Render, ScrollDelta, ScrollHandle, ScrollWheelEvent,
        StatefulInteractiveElement, Styled, TestAppContext, TouchPhase, UniformListScrollHandle, Window, div, point, px, size,
    };

    use super::super::*;
    use crate::theme::{Appearance, set_appearance};

    struct Host {
        panel: ScrollHandle,
        inner: UniformListScrollHandle,
        rows: usize,
        status: FileDiffStatus,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let lines = (0..self.rows)
                .map(|n| DiffLine { kind: DiffLineKind::Added, old_line: None, new_line: Some(n as u32 + 1), text: format!("line {n}").into() })
                .collect();
            div()
                .id("panel")
                .w(px(600.))
                .h(px(400.))
                .overflow_y_scroll()
                .track_scroll(&self.panel)
                .child(div().h(px(100.)))
                .child(FileDiff::new("wheel", "src/main.rs", lines).status(self.status).scroll_handle(self.inner.clone()).preview_rows(8))
                .child(div().h(px(1500.)))
        }
    }

    type Opened<'a> = (gpui_kit::Entity<Host>, &'a mut gpui_kit::VisualTestContext, ScrollHandle, UniformListScrollHandle);

    fn open(status: FileDiffStatus, cx: &mut TestAppContext) -> Opened<'_> {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Light, cx);
            cx.set_reduce_motion(true);
        });
        let (panel, inner) = (ScrollHandle::new(), UniformListScrollHandle::new());
        let handles = (panel.clone(), inner.clone());
        let (host, cx) = cx.add_window_view(move |_, _| Host { panel, inner, rows: 40, status });
        cx.simulate_resize(size(px(700.), px(500.)));
        settle(&host, cx);
        (host, cx, handles.0, handles.1)
    }

    fn settle(host: &gpui_kit::Entity<Host>, cx: &mut gpui_kit::VisualTestContext) {
        for _ in 0..4 {
            cx.run_until_parked();
            host.update(cx, |_, cx| cx.notify());
        }
        cx.run_until_parked();
    }

    /// A wheel of `dy` px over the diff's rows: negative moves on, positive goes back.
    fn wheel(host: &gpui_kit::Entity<Host>, cx: &mut gpui_kit::VisualTestContext, dy: f32) {
        let at = cx.debug_bounds("diff-body").expect("the diff is drawn").center();
        cx.simulate_event(ScrollWheelEvent {
            position: at,
            delta: ScrollDelta::Pixels(point(px(0.), px(dy))),
            modifiers: Modifiers::none(),
            touch_phase: TouchPhase::Moved,
        });
        settle(host, cx);
    }

    fn press(host: &gpui_kit::Entity<Host>, cx: &mut gpui_kit::VisualTestContext) {
        let at = cx.debug_bounds("diff-body").expect("the diff is drawn").origin + point(px(40.), px(10.));
        cx.simulate_click(at, Modifiers::none());
        settle(host, cx);
    }

    fn top(panel: &ScrollHandle) -> f32 {
        -f32::from(panel.offset().y)
    }

    /// How far the opened rows are from their end, in px.
    fn from_end(inner: &UniformListScrollHandle) -> f32 {
        crate::scroll_chain::from_end(&inner.0.borrow().base_handle)
    }

    fn inner_top(inner: &UniformListScrollHandle) -> f32 {
        -f32::from(inner.0.borrow().base_handle.offset().y)
    }

    #[gpui_kit::test]
    fn a_clipped_diff_hands_the_wheel_to_the_panel(cx: &mut TestAppContext) {
        let (host, cx, panel, _) = open(FileDiffStatus::Complete, cx);
        wheel(&host, cx, -60.);
        assert!(top(&panel) > 50., "the panel scrolled: {}", top(&panel));
    }

    #[gpui_kit::test]
    fn an_opened_diff_scrolls_its_own_rows_before_the_panel(cx: &mut TestAppContext) {
        let (host, cx, panel, inner) = open(FileDiffStatus::Complete, cx);
        press(&host, cx);
        wheel(&host, cx, -60.);
        assert!(top(&panel) < 1., "the rows took the wheel: {}", top(&panel));
        assert!(inner_top(&inner) > 50., "and moved: {}", inner_top(&inner));
    }

    #[gpui_kit::test]
    fn a_diff_folded_back_hands_the_wheel_to_the_panel_again(cx: &mut TestAppContext) {
        let (host, cx, panel, _) = open(FileDiffStatus::Complete, cx);
        press(&host, cx);
        wheel(&host, cx, -60.);
        press(&host, cx);
        wheel(&host, cx, -60.);
        assert!(top(&panel) > 50., "the panel scrolled: {}", top(&panel));
    }

    #[gpui_kit::test]
    fn an_opened_diff_starts_at_its_top_each_time(cx: &mut TestAppContext) {
        let (host, cx, _, inner) = open(FileDiffStatus::Complete, cx);
        press(&host, cx);
        wheel(&host, cx, -100.);
        assert!(inner_top(&inner) > 50.);
        press(&host, cx);
        press(&host, cx);
        assert!(inner_top(&inner) < 1., "not where it was left: {}", inner_top(&inner));
    }

    #[gpui_kit::test]
    fn a_streaming_diff_follows_its_newest_row_until_the_reader_scrolls_up(cx: &mut TestAppContext) {
        let (host, cx, _, inner) = open(FileDiffStatus::Streaming, cx);
        press(&host, cx);
        assert!(from_end(&inner) < 1., "pinned to the newest row: {}", from_end(&inner));
        host.update(cx, |h, cx| {
            h.rows = 60;
            cx.notify();
        });
        settle(&host, cx);
        assert!(from_end(&inner) < 1., "it follows the new rows: {}", from_end(&inner));
        wheel(&host, cx, 200.);
        let away = from_end(&inner);
        assert!(away > 150., "the reader scrolled up: {away}");
        host.update(cx, |h, cx| {
            h.rows = 80;
            cx.notify();
        });
        settle(&host, cx);
        assert!(from_end(&inner) > away - 1., "it does not pull them back: {} from {away}", from_end(&inner));
    }

    #[gpui_kit::test]
    fn a_streaming_diff_follows_again_when_the_reader_comes_back_to_the_end(cx: &mut TestAppContext) {
        let (host, cx, _, inner) = open(FileDiffStatus::Streaming, cx);
        press(&host, cx);
        wheel(&host, cx, 200.);
        wheel(&host, cx, -4000.);
        assert!(from_end(&inner) < 1.);
        host.update(cx, |h, cx| {
            h.rows = 80;
            cx.notify();
        });
        settle(&host, cx);
        assert!(from_end(&inner) < 1., "following again: {}", from_end(&inner));
    }
}

/// The card clips to a rectangle, so the row at its bottom edge rounds its own corners: only when a row is there.
#[test]
fn the_last_row_rounds_its_corners_when_it_stands_at_the_cards_bottom() {
    use super::helpers::bottom_row;
    assert_eq!(bottom_row(3, false, false, false), Some(2), "the rows end the card");
    assert_eq!(bottom_row(0, false, false, false), None, "no rows");
    assert_eq!(bottom_row(3, true, false, false), None, "the hint stands under the rows");
    assert_eq!(bottom_row(3, false, true, false), None, "the copy footer stands under the rows");
    assert_eq!(bottom_row(30, false, false, true), None, "scrolling rows are cut anywhere");
}
