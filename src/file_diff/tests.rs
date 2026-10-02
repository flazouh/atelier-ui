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
