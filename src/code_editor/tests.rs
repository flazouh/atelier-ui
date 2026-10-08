use super::*;

#[test]
fn a_path_picks_its_language_by_extension() {
    let cases = [
        ("crates/beui/src/hunk.rs", Some("rust")),
        ("src/app.tsx", Some("tsx")),
        ("src/index.ts", Some("typescript")),
        ("main.go", Some("go")),
        ("Cargo.toml", Some("toml")),
        ("README.md", Some("markdown")),
        ("data.json", Some("json")),
        ("ci.yml", Some("yaml")),
    ];
    for (path, want) in cases {
        assert_eq!(language_for(path), want, "{path}");
    }
}

#[test]
fn a_path_with_no_known_extension_stays_plain() {
    assert_eq!(language_for("LICENSE"), None);
    assert_eq!(language_for("notes.xyz"), None);
    // A whole name with no grammar here; Makefile has one (`make`).
    assert_eq!(language_for("Dockerfile"), None);
}

#[test]
fn every_language_we_claim_is_one_the_build_turns_on() {
    // A name in LANGUAGES with no tree-sitter feature would highlight nothing, so the two lists must
    // agree. Cargo.toml is the source of truth.
    let manifest = include_str!("../../Cargo.toml");
    for language in LANGUAGES {
        // JSON rides along with the base tree-sitter feature and has no name of its own.
        if *language == "json" {
            assert!(
                manifest.contains("\"tree-sitter\""),
                "the base tree-sitter feature carries json"
            );
            continue;
        }
        let feature = format!("\"tree-sitter-{language}\"");
        assert!(
            manifest.contains(&feature),
            "{language} is claimed but {feature} is not on"
        );
    }
}

#[test]
fn both_syntax_themes_parse_and_carry_our_muted_tokens() {
    for appearance in [Appearance::Dark, Appearance::Light] {
        let theme = syntax_theme(appearance);
        let keyword = theme
            .style
            .syntax
            .style("keyword")
            .expect("keywords are colored");
        let comment = theme
            .style
            .syntax
            .style("comment")
            .expect("comments are colored");
        assert!(keyword.color.is_some());
        assert_ne!(
            keyword.color, comment.color,
            "a keyword must not read as a comment"
        );
    }
}

#[test]
fn a_severity_picks_a_mark_that_matches_how_loud_it_is() {
    use crate::theme::Theme;
    let theme = Theme::dark();
    let page = theme.background;
    let loud = crate::theme::contrast(
        theme.status_tone(severity_tone(DiagnosticSeverity::Error)),
        page,
    );
    let quiet = crate::theme::contrast(
        theme.status_tone(severity_tone(DiagnosticSeverity::Hint)),
        page,
    );
    assert!(loud > quiet, "an error must read louder than a hint");
}

#[test]
fn every_severity_maps_to_its_own_mark() {
    let all = [
        DiagnosticSeverity::Error,
        DiagnosticSeverity::Warning,
        DiagnosticSeverity::Info,
        DiagnosticSeverity::Hint,
    ];
    let tones: Vec<_> = all.iter().map(|s| severity_tone(*s)).collect();
    for (i, tone) in tones.iter().enumerate() {
        for (j, other) in tones.iter().enumerate() {
            if i != j {
                assert_ne!(tone, other, "{:?} and {:?} share a mark", all[i], all[j]);
            }
        }
    }
}

/// Every kind of edit takes the background parse, and once it lands the editor's colours are those of a
/// fresh parse of the same text.
mod background {
    use gpui_kit::{
        ClipboardItem, Context, Entity, IntoElement, ParentElement, Render, Styled, TestAppContext,
        VisualTestContext, Window,
        component::{
            highlighter::SyntaxHighlighter,
            input::{EditorState, Paste, Redo, Undo},
        },
        div, px,
    };

    use crate::{
        code_editor::CodeEditor,
        inline_review::{Decision, InlineHunk, apply},
        theme::{Appearance, set_appearance},
    };

    /// Old rows 1..2 above new rows 2..4, as a review shows them.
    const TEXT: &str =
        "fn a() {\n    let old = 1;\n    let s = format!(\"{}\", 2);\n    vec![3];\n}\n";

    struct View {
        editor: Entity<EditorState>,
    }

    impl Render for View {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .child(CodeEditor::new(&self.editor).height(px(400.)))
        }
    }

    fn open(cx: &mut TestAppContext) -> (Entity<EditorState>, &mut VisualTestContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
        });
        let (view, cx) = cx.add_window_view(|window, cx| View {
            editor: CodeEditor::state("t.rs", TEXT, window, cx),
        });
        let editor = cx.update(|window, cx| {
            let editor = view.read(cx).editor.clone();
            editor.update(cx, |state, cx| state.focus(window, cx));
            editor
        });
        cx.run_until_parked();
        (editor, cx)
    }

    /// Lets the background parse land, then compares the editor's colours with a fresh parse.
    #[track_caller]
    fn assert_colours_are_a_fresh_parse(
        editor: &Entity<EditorState>,
        cx: &mut VisualTestContext,
        what: &str,
    ) {
        cx.run_until_parked();
        let (shown, fresh) = cx.update(|_, cx| {
            let theme = gpui_kit::component::Theme::global(cx)
                .highlight_theme
                .clone();
            let state = editor.read(cx);
            let text = state.value().to_string();
            let range = 0..text.len();
            let shown = state
                .syntax_styles(&range, &*theme)
                .expect("a Rust editor has a highlighter");
            let mut fresh = SyntaxHighlighter::new("rust");
            fresh.update(
                None,
                &gpui_kit::base::input::Rope::from(text.as_str()),
                None,
            );
            (shown, fresh.styles(&range, &*theme))
        });
        assert_eq!(shown, fresh, "{what}");
    }

    fn hunk() -> InlineHunk {
        InlineHunk::new("h", 1..2, 2..4)
    }

    #[gpui_kit::test]
    fn after_accepting_a_hunk(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx);
        cx.update(|window, cx| apply(&editor, &[(hunk(), Decision::Accept)], window, cx));
        assert!(
            !editor.read_with(cx, |s, _| s.value().contains("old")),
            "the old row went"
        );
        assert_colours_are_a_fresh_parse(&editor, cx, "accept");
    }

    #[gpui_kit::test]
    fn after_rejecting_a_hunk(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx);
        cx.update(|window, cx| apply(&editor, &[(hunk(), Decision::Reject)], window, cx));
        assert!(
            !editor.read_with(cx, |s, _| s.value().contains("format!")),
            "the new rows went"
        );
        assert_colours_are_a_fresh_parse(&editor, cx, "reject");
    }

    #[gpui_kit::test]
    fn after_undo_and_redo(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx);
        cx.simulate_input("m!(9); ");
        cx.run_until_parked();
        let typed = editor.read_with(cx, |s, _| s.value().to_string());
        cx.update(|window, cx| window.dispatch_action(Box::new(Undo), cx));
        assert_eq!(
            editor.read_with(cx, |s, _| s.value().to_string()),
            TEXT,
            "undo takes the typing back"
        );
        assert_colours_are_a_fresh_parse(&editor, cx, "undo");
        cx.update(|window, cx| window.dispatch_action(Box::new(Redo), cx));
        assert_eq!(
            editor.read_with(cx, |s, _| s.value().to_string()),
            typed,
            "redo puts it again"
        );
        assert_colours_are_a_fresh_parse(&editor, cx, "redo");
    }

    #[gpui_kit::test]
    fn after_a_paste(cx: &mut TestAppContext) {
        let (editor, cx) = open(cx);
        cx.write_to_clipboard(ClipboardItem::new_string(
            "fn z() { m!(\"pasted\"); }\n".into(),
        ));
        cx.update(|window, cx| window.dispatch_action(Box::new(Paste), cx));
        assert!(
            editor.read_with(cx, |s, _| s.value().contains("pasted")),
            "the paste went in"
        );
        assert_colours_are_a_fresh_parse(&editor, cx, "paste");
    }
}
