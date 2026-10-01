use super::*;
use crate::{code_editor::syntax_theme, file_diff::DiffLine, theme::Appearance};

#[test]
fn each_file_name_picks_its_language() {
    let cases = [
        ("src/main.rs", Some("rust")),
        ("server.zig", Some("zig")),
        ("App.tsx", Some("tsx")),
        ("index.d.ts", Some("typescript")),
        ("run.sh", Some("bash")),
        ("Makefile", Some("make")),
        ("CMakeLists.txt", Some("cmake")),
        ("schema.graphql", Some("graphql")),
        ("Widget.svelte", Some("svelte")),
        ("page.astro", Some("astro")),
        ("lib.ex", Some("elixir")),
        ("a.proto", Some("proto")),
        ("fix.diff", Some("diff")),
        ("query.sql", Some("sql")),
        ("Main.kt", None),
        ("App.swift", Some("swift")),
        ("main.c", Some("c")),
        ("main.cpp", Some("cpp")),
        ("tests.rs:40", Some("rust")),
        ("notes.zzz", None),
    ];
    for (path, want) in cases {
        assert_eq!(language_for(path), want, "{path}");
    }
}

#[test]
fn every_language_named_has_a_grammar_built_in() {
    // Without its grammar a highlighter falls back to plain text and says so.
    let missing: Vec<_> = LANGUAGES.iter().filter(|l| SyntaxHighlighter::new(l).language().as_ref() != **l).collect();
    assert!(missing.is_empty(), "no grammar for {missing:?}");
}

/// The colour of the text at `needle` on line `line` of `lines`.
fn colour_at(lines: &[LineRuns], text: &str, line: usize, needle: &str) -> Option<gpui_kit::Hsla> {
    let row = text.split('\n').nth(line).unwrap();
    let at = row.find(needle).unwrap();
    lines[line].iter().find(|(r, _)| r.contains(&at)).and_then(|(_, s)| s.color)
}

#[test]
fn a_comment_across_two_lines_is_a_comment_on_both() {
    let text = "fn a() {}\n/* one\n   two */\nfn b() {}";
    let theme = syntax_theme(Appearance::Dark);
    let lines = compute("rust", text, &theme);
    let comment = colour_at(&lines, text, 1, "one");
    assert!(comment.is_some());
    assert_eq!(colour_at(&lines, text, 2, "two"), comment, "the second line is still the comment");
    assert_ne!(colour_at(&lines, text, 3, "fn"), comment, "and the code after it is not");
}

#[test]
fn a_diff_highlights_each_side_as_its_whole_text() {
    // The comment opens on a removed line and closes on a context line, so only its old side has it.
    let diff = "@@ -1,3 +1,3 @@\n fn a() {}\n-/* old\n+// new\n    still */\n fn b() {}";
    let lines = DiffLine::parse(diff);
    let (old, new) = sides(&lines);
    assert_eq!(old.text, "fn a() {}\n/* old\n   still */\nfn b() {}");
    assert_eq!(new.text, "fn a() {}\n// new\n   still */\nfn b() {}");
    let theme = syntax_theme(Appearance::Dark);
    let old_runs = compute("rust", &old.text, &theme);
    let comment = colour_at(&old_runs, &old.text, 1, "old");
    assert_eq!(colour_at(&old_runs, &old.text, 2, "still"), comment, "on the old side the context line is inside the comment");
    // Each diff row maps to its side's line: a removed row to the old side, an added one to the new.
    let map = row_sides(&lines);
    assert_eq!(map[2], Some((Side::Old, 1)));
    assert_eq!(map[3], Some((Side::New, 1)));
    assert_eq!(map[4], Some((Side::New, 2)), "a context row reads from the new side");
    assert_eq!(map[0], None, "the hunk header is not code");
}

#[test]
fn the_cache_computes_once_per_text() {
    let mut cache = SyntaxCache::default();
    let theme = syntax_theme(Appearance::Dark);
    let key = Key::new("rust", "fn a() {}", Appearance::Dark);
    let first = cache.get_or_compute(key.clone(), || compute("rust", "fn a() {}", &theme));
    let second = cache.get_or_compute(key, || panic!("the second render must not parse"));
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(cache.computed, 1);
    // Another text, or the same text in the other theme, is another entry.
    cache.get_or_compute(Key::new("rust", "fn b() {}", Appearance::Dark), || compute("rust", "fn b() {}", &theme));
    cache.get_or_compute(Key::new("rust", "fn a() {}", Appearance::Light), || compute("rust", "fn a() {}", &theme));
    assert_eq!(cache.computed, 3);
}

#[test]
fn a_warm_highlighter_colours_a_new_text_as_a_fresh_one_does() {
    // The same macro at the same bytes, with another body: a reused injection tree would colour the second
    // body as the first.
    let first = "fn a() { m!(x + 1) }";
    let second = "fn a() { m!(\"a\" 1) }";
    let theme = syntax_theme(Appearance::Dark);
    compute("rust", first, &theme);
    let warm = compute("rust", second, &theme);
    let fresh = compute_with(&mut SyntaxHighlighter::new("rust"), second, &theme);
    assert_eq!(warm, fresh);
}

#[test]
fn the_cache_forgets_its_oldest_text_past_its_size() {
    let mut cache = SyntaxCache::default();
    for i in 0..=MAX_ENTRIES {
        cache.get_or_compute(Key::new("rust", &i.to_string(), Appearance::Dark), Vec::new);
    }
    assert_eq!(cache.entries.len(), MAX_ENTRIES);
    assert!(cache.get(&Key::new("rust", "0", Appearance::Dark)).is_none(), "the first text went first");
    assert!(cache.get(&Key::new("rust", &MAX_ENTRIES.to_string(), Appearance::Dark)).is_some());
}

#[test]
fn a_parsing_slot_keeps_the_colours_of_its_unchanged_lines() {
    let theme = syntax_theme(Appearance::Dark);
    let last_text = "fn a() {}\nfn b() {}";
    let last = compute("rust", last_text, &theme);
    let shown = carry(last_text, &last, "fn a() {}\nfn c() {}\nfn d");
    assert_eq!(shown[0], last[0], "the same line keeps its colours");
    assert!(shown[1].is_empty(), "a changed line draws plain");
    assert!(shown[2].is_empty(), "and so does a new one");
}

#[test]
fn the_slots_are_as_many_as_the_texts_the_cache_keeps() {
    let mut cache = SyntaxCache::default();
    for i in 0..=MAX_ENTRIES {
        cache.generation(ElementId::Integer(i as u64));
    }
    assert_eq!(cache.generations.len(), MAX_ENTRIES);
    assert!(!cache.generations.contains_key(&ElementId::Integer(0)), "the oldest slot went first");
    let again = cache.generation(ElementId::Integer(MAX_ENTRIES as u64));
    assert!(Arc::ptr_eq(&again, &cache.generation(ElementId::Integer(MAX_ENTRIES as u64))), "a slot keeps its generation");
}

mod app {
    use gpui_kit::{ElementId, TestAppContext};

    use super::super::*;
    use crate::theme::{Appearance, set_appearance};

    fn setup(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
        });
    }

    fn slot(name: &'static str) -> ElementId {
        ElementId::Name(name.into())
    }

    /// A text parses off the UI thread, and draws from the cache once it lands.
    #[gpui_kit::test]
    fn a_text_parses_off_the_ui_thread_then_draws_from_the_cache(cx: &mut TestAppContext) {
        setup(cx);
        assert!(cx.update(|cx| highlight("lua", "local a = 1", slot("a"), cx)).is_none(), "nothing to show yet");
        cx.run_until_parked();
        let landed = cx.update(|cx| highlight("lua", "local a = 1", slot("a"), cx)).expect("the parse landed");
        let again = cx.update(|cx| highlight("lua", "local a = 1", slot("a"), cx)).unwrap();
        assert!(Arc::ptr_eq(&landed, &again), "and is not parsed again");
    }

    /// A streamed block: while its longer text parses, its first line keeps its colours.
    #[gpui_kit::test]
    fn a_streamed_block_keeps_its_colours_between_tokens(cx: &mut TestAppContext) {
        setup(cx);
        let first = "def f(v):\n    return v";
        cx.update(|cx| highlight("python", first, slot("b"), cx));
        cx.run_until_parked();
        let done = cx.update(|cx| highlight("python", first, slot("b"), cx)).unwrap();
        let next = format!("{first} + 1");
        let between = cx.update(|cx| highlight("python", &next, slot("b"), cx)).expect("the slot's last colours");
        assert_eq!(between[0], done[0]);
        assert!(!between[0].is_empty());
        cx.run_until_parked();
        let landed = cx.update(|cx| highlight("python", &next, slot("b"), cx)).unwrap();
        assert!(!landed[1].is_empty(), "the changed line has its colours once the parse lands");
    }

    /// Twenty blocks at once: none parses on the UI thread, and all land after the frame.
    #[gpui_kit::test]
    fn many_blocks_in_one_frame_all_land(cx: &mut TestAppContext) {
        setup(cx);
        let blocks: Vec<String> = (0..20).map(|i| format!("def f{i}(v):\n    return v + {i}\n").repeat(20)).collect();
        let slots: Vec<ElementId> = (0..20).map(ElementId::Integer).collect();
        let first: Vec<_> =
            cx.update(|cx| blocks.iter().zip(&slots).map(|(b, s)| highlight("python", b, s.clone(), cx).is_some()).collect());
        assert!(first.iter().all(|&drawn| !drawn), "the frame parses none of them");
        cx.run_until_parked();
        let later: Vec<_> =
            cx.update(|cx| blocks.iter().zip(&slots).map(|(b, s)| highlight("python", b, s.clone(), cx).is_some()).collect());
        assert!(later.iter().all(|&drawn| drawn), "all landed after it");
    }

    /// A streamed block: 50 versions in a row. Each new one drops the parse of the one before, so
    /// only the last parses (and at most one already running), and the last one lands.
    #[gpui_kit::test]
    fn a_new_text_drops_the_parse_of_the_one_before(cx: &mut TestAppContext) {
        setup(cx);
        let versions: Vec<String> = (1..=50).map(|n| (0..n).map(|i| format!("x{i} = {i}\n")).collect()).collect();
        for text in &versions {
            cx.update(|cx| highlight("python", text, slot("stream"), cx));
        }
        cx.run_until_parked();
        let parsed = cx.update(|cx| cx.global::<SyntaxCache>().parsed.load(std::sync::atomic::Ordering::Relaxed));
        assert!(parsed <= 2, "{parsed} of 50 versions parsed");
        assert!(cx.update(|cx| highlight("python", versions.last().unwrap(), slot("stream"), cx)).is_some(), "the last one landed");
    }
}
