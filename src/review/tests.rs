use super::*;
use crate::inline_review::{Decision, InlineHunk, apply_to_text};

#[test]
fn the_bar_counts_what_changed_and_what_is_reviewed() {
    let p = ReviewProgress {
        files: 7,
        reviewed: 3,
        added: 62,
        removed: 12,
    };
    assert_eq!(p.changed_text(), "7 changed");
    assert_eq!(p.reviewed_text(), "3 of 7 reviewed");
    assert_eq!(p.reviewed_text_as("seen"), "3 of 7 seen");
    assert!((p.fraction() - 3. / 7.).abs() < 1e-6);
    assert!(!p.is_done());
}

#[test]
fn every_file_reviewed_reads_all_and_next_becomes_done() {
    let p = ReviewProgress {
        files: 7,
        reviewed: 7,
        added: 62,
        removed: 12,
    };
    assert_eq!(p.reviewed_text(), "All 7 reviewed");
    assert!(p.is_done());
    assert_eq!(p.next_label(), "Done");
    assert_eq!(ReviewProgress { reviewed: 6, ..p }.next_label(), "Next");
}

#[test]
fn nothing_to_review_is_not_done_and_has_no_progress() {
    let p = ReviewProgress::default();
    assert_eq!(p.fraction(), 0.);
    assert!(!p.is_done());
}

#[test]
fn next_and_previous_walk_the_file_order_and_stop_at_the_ends() {
    let order: Vec<SharedString> = ["a", "b", "c"].map(SharedString::from).to_vec();
    assert_eq!(step(&order, Some(&"a".into()), 1), Some("b".into()));
    assert_eq!(step(&order, Some(&"c".into()), 1), None);
    assert_eq!(step(&order, Some(&"b".into()), -1), Some("a".into()));
    assert_eq!(step(&order, Some(&"a".into()), -1), None);
    // With no current file, Next starts at the top and Previous at the bottom.
    assert_eq!(step(&order, None, 1), Some("a".into()));
    assert_eq!(step(&order, None, -1), Some("c".into()));
    assert_eq!(step(&[], None, 1), None);
}

#[test]
fn accepting_a_file_keeps_the_new_side_of_every_hunk() {
    // Rows: 0 keep, 1 old, 2 new, 3 keep, 4 old, 5 new, 6 new, 7 keep.
    let text = "keep\nold one\nnew one\nkeep\nold two\nnew two\nnew three\nkeep\n";
    let hunks = [
        InlineHunk::new("a", 1..2, 2..3),
        InlineHunk::new("b", 4..5, 5..7),
    ];
    let accepted = apply_to_text(text, &whole_file(&hunks, Decision::Accept));
    assert_eq!(accepted, "keep\nnew one\nkeep\nnew two\nnew three\nkeep\n");
    let rejected = apply_to_text(text, &whole_file(&hunks, Decision::Reject));
    assert_eq!(rejected, "keep\nold one\nkeep\nold two\nkeep\n");
}

mod keys {
    use std::{cell::RefCell, rc::Rc};

    use gpui_kit::{
        Context, Entity, FocusHandle, IntoElement, ParentElement, Render, Styled, TestAppContext,
        Window, component::input::EditorState, div,
    };

    use super::super::ReviewHandlers;
    use crate::{
        code_editor::CodeEditor,
        theme::{Appearance, set_appearance},
    };

    /// A review pane with an editor in it, logging the commands that reach it.
    struct Pane {
        focus: FocusHandle,
        editor: Entity<EditorState>,
        read_only: bool,
        log: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Render for Pane {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let log = |word: &'static str| {
                let log = self.log.clone();
                move |_: &mut Window, _: &mut gpui_kit::App| log.borrow_mut().push(word)
            };
            let handlers = ReviewHandlers::default()
                .on_next(log("next"))
                .on_previous(log("previous"))
                .on_mark(log("mark"))
                .on_review_mode(log("review mode"))
                .on_toggle_files(log("files"))
                .on_toggle_details(log("details"))
                .on_dismiss(log("dismiss"));
            handlers.keys(
                div()
                    .size_full()
                    .child(CodeEditor::new(&self.editor).read_only(self.read_only)),
                &self.focus,
            )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
    ) -> (
        Entity<Pane>,
        &mut gpui_kit::VisualTestContext,
        Rc<RefCell<Vec<&'static str>>>,
    ) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            crate::code_editor::bind_keys(cx);
            crate::inline_review::bind_keys(cx);
            super::super::bind_keys(cx);
            set_appearance(Appearance::Light, cx);
        });
        let log = Rc::new(RefCell::new(Vec::new()));
        let seen = log.clone();
        let (pane, cx) = cx.add_window_view(move |window, cx| Pane {
            focus: cx.focus_handle(),
            editor: CodeEditor::state("x.rs", "", window, cx),
            read_only: false,
            log: seen,
        });
        (pane, cx, log)
    }

    /// GitQuiet's letters reach the review while the pane has focus.
    #[gpui_kit::test]
    fn s_w_x_and_r_reach_their_commands_from_the_pane(cx: &mut TestAppContext) {
        let (pane, cx, log) = setup(cx);
        cx.update(|window, cx| pane.read(cx).focus.clone().focus(window, cx));
        cx.simulate_keystrokes("s s w x r");
        assert_eq!(
            *log.borrow(),
            ["next", "next", "previous", "mark", "review mode"]
        );
        // A shifted combo reads the same whichever way the platform reports the letter.
        log.borrow_mut().clear();
        cx.simulate_keystrokes("secondary-shift-b");
        cx.simulate_keystrokes("secondary-shift-B");
        assert_eq!(*log.borrow(), ["files", "files"]);
    }

    /// In the editor the letters are text, and Escape hands the letters back to the pane.
    #[gpui_kit::test]
    fn s_typed_in_the_editor_is_text_until_escape(cx: &mut TestAppContext) {
        let (pane, cx, log) = setup(cx);
        cx.update(|window, cx| {
            let editor = pane.read(cx).editor.clone();
            editor.update(cx, |state, cx| state.focus(window, cx));
        });
        cx.simulate_keystrokes("s w x r");
        let text = pane.read_with(cx, |p, cx| p.editor.read(cx).value().to_string());
        assert_eq!(text, "swxr");
        assert!(log.borrow().is_empty(), "nothing reached the review");
        cx.simulate_keystrokes("escape s");
        assert_eq!(
            *log.borrow(),
            ["next"],
            "after Escape the pane has the letters"
        );
        let text = pane.read_with(cx, |p, cx| p.editor.read(cx).value().to_string());
        assert_eq!(text, "swxr", "and the editor took no more");
    }

    /// A chord with Command never types, so it reaches the review from the editor too.
    #[gpui_kit::test]
    fn command_b_works_from_the_editor_and_types_nothing(cx: &mut TestAppContext) {
        let (pane, cx, log) = setup(cx);
        cx.update(|window, cx| {
            let editor = pane.read(cx).editor.clone();
            editor.update(cx, |state, cx| state.focus(window, cx));
        });
        cx.simulate_keystrokes("a secondary-b");
        assert_eq!(*log.borrow(), ["details"]);
        let text = pane.read_with(cx, |p, cx| p.editor.read(cx).value().to_string());
        assert_eq!(text, "a", "the editor took nothing from the chord");
    }

    /// A read-only editor, such as a pull request's diff, is read rather than typed in, so the letters
    /// reach the review from it.
    #[gpui_kit::test]
    fn letters_reach_the_review_from_a_read_only_editor(cx: &mut TestAppContext) {
        let (pane, cx, log) = setup(cx);
        cx.update(|window, cx| {
            pane.update(cx, |p, cx| {
                p.read_only = true;
                cx.notify();
            });
            let editor = pane.read(cx).editor.clone();
            editor.update(cx, |state, cx| state.focus(window, cx));
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("s w");
        assert_eq!(*log.borrow(), ["next", "previous"]);
        let text = pane.read_with(cx, |p, cx| p.editor.read(cx).value().to_string());
        assert_eq!(text, "", "and the editor took none of them");
        // Nothing is being typed, so one Escape is the review's own.
        cx.simulate_keystrokes("escape");
        assert_eq!(log.borrow().last(), Some(&"dismiss"));
    }
}
/// One file reads "Reviewed", not "All 1 reviewed"; more read "All 3 reviewed".
#[test]
fn one_reviewed_file_reads_reviewed() {
    let one = ReviewProgress {
        files: 1,
        reviewed: 1,
        added: 1,
        removed: 0,
    };
    assert_eq!(one.reviewed_text(), "Reviewed");
    assert_eq!(one.reviewed_text_as("seen"), "Seen");
    assert_eq!(one.short_text("seen"), "Seen");
    assert_eq!(
        ReviewProgress { reviewed: 0, ..one }.reviewed_text(),
        "0 of 1 reviewed"
    );
    assert_eq!(
        ReviewProgress {
            files: 3,
            reviewed: 3,
            added: 1,
            removed: 0
        }
        .reviewed_text(),
        "All 3 reviewed"
    );
}
