//! Syntax highlighting timings, against the targets in `docs/code-editor.md` ("Performance").
//! Run in release, on the machine you report for, one test thread so nothing else competes:
//!     cargo test --release -p beui --test highlight_bench -- --ignored --nocapture --test-threads=1
//! Every fixture is generated here. Each number is the median and the 95th percentile of 20 runs.
//! The frame side (what a scroll costs on the UI thread) is the gallery's "Highlight load" story.

use std::{hint::black_box, ops::Range, time::{Duration, Instant}};

use atelier_ui::{
    code_editor::syntax_theme,
    file_diff::DiffLine,
    syntax::{Key, Side, compute, row_sides, sides},
    theme::Appearance,
};
use gpui_kit::{
    base::input::Rope,
    component::highlighter::SyntaxHighlighter,
};
use tree_sitter::{InputEdit, Point};

const RUNS: usize = 20;
/// About a screen of rows.
const VISIBLE_ROWS: usize = 50;

fn rust_file(lines: usize) -> String {
    let mut out = String::new();
    let mut i = 0;
    while out.lines().count() < lines {
        out.push_str(&format!(
            "/// Adds {i} to the input, and says so.\n\
             pub fn add_{i}(value: u64) -> Result<u64, String> {{\n\
             \x20   let label = \"step {i}: a string with \\\"quotes\\\" in it\";\n\
             \x20   /* a block comment\n\
             \x20      across two lines */\n\
             \x20   match value.checked_add({i}) {{\n\
             \x20       Some(sum) => Ok(sum),\n\
             \x20       None => Err(format!(\"{{label}} overflowed at {{}}\", value)),\n\
             \x20   }}\n\
             }}\n\n"
        ));
        i += 1;
    }
    out.lines().take(lines).collect::<Vec<_>>().join("\n")
}

/// `rust_file`'s functions, 50 to a module: the same text, with about 19 items at the top level
/// rather than about 900.
fn rust_in_modules(lines: usize) -> String {
    let flat = rust_file(lines);
    let functions: Vec<&str> = flat.split("\n/// Adds ").collect();
    let mut out = String::new();
    for (i, group) in functions.chunks(50).enumerate() {
        out.push_str(&format!("mod m{i} {{\n"));
        for (j, function) in group.iter().enumerate() {
            if i + j > 0 {
                out.push_str("\n/// Adds ");
            }
            out.push_str(function);
        }
        out.push_str("\n}\n");
    }
    out
}

fn ts_file(lines: usize) -> String {
    let mut out = Vec::new();
    for i in 0.. {
        if out.len() >= lines {
            break;
        }
        out.extend([
            format!("/** Adds {i}, and says so. */"),
            format!("export function add{i}(value: number): number {{"),
            format!("  const label = `step ${{value}} of {i}`;"),
            "  // a line comment".to_string(),
            format!("  return value + {i} /* inline */;"),
            "}".to_string(),
            String::new(),
        ]);
    }
    out.truncate(lines);
    out.join("\n")
}

fn zig_file(lines: usize) -> String {
    let mut out = Vec::new();
    for i in 0.. {
        if out.len() >= lines {
            break;
        }
        out.extend([
            format!("/// Adds {i}."),
            format!("pub fn add{i}(value: u64) !u64 {{"),
            format!("    const label = \"step {i}\";"),
            "    _ = label;".to_string(),
            format!("    return value + {i};"),
            "}".to_string(),
            String::new(),
        ]);
    }
    out.truncate(lines);
    out.join("\n")
}

/// A unified diff of `lines` rows over a Rust file: hunks of context, removed and added rows.
fn diff(lines: usize) -> String {
    let source: Vec<String> = rust_file(lines).lines().map(str::to_string).collect();
    let mut out = vec![format!("@@ -1,{lines} +1,{lines} @@")];
    for (i, line) in source.iter().enumerate() {
        out.push(match i % 12 {
            3 => format!("-{line}"),
            4 => format!("+{line} // changed"),
            _ => format!(" {line}"),
        });
    }
    out.truncate(lines + 1);
    out.join("\n")
}

fn time(mut f: impl FnMut()) -> (Duration, Duration) {
    let mut samples: Vec<Duration> = (0..RUNS)
        .map(|_| {
            let at = Instant::now();
            f();
            at.elapsed()
        })
        .collect();
    samples.sort();
    (samples[RUNS / 2], samples[(RUNS * 95).div_ceil(100) - 1])
}

fn report(name: &str, target: &str, (median, p95): (Duration, Duration)) {
    println!("{name:<58} median {:>9.3} ms   p95 {:>9.3} ms   target {target}", ms(median), ms(p95));
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.
}

/// The byte range of `rows` rows from `first`.
fn rows_range(text: &str, first: usize, rows: usize) -> Range<usize> {
    let starts: Vec<usize> = std::iter::once(0).chain(text.match_indices('\n').map(|(i, _)| i + 1)).collect();
    let start = starts.get(first).copied().unwrap_or(text.len());
    let end = starts.get(first + rows).map_or(text.len(), |e| e - 1);
    start..end
}

fn go_file(lines: usize) -> String {
    let mut out = Vec::new();
    for i in 0.. {
        if out.len() >= lines {
            break;
        }
        out.extend([
            format!("// F{i} adds {i}."),
            format!("func F{i}(v int) int {{"),
            format!("\tlabel := \"step {i}\""),
            "\t_ = label".to_string(),
            format!("\treturn v + {i}"),
            "}".to_string(),
            String::new(),
        ]);
    }
    out.truncate(lines);
    out.join("\n")
}

/// A highlighter that has parsed `text`, and 20 one-character edits at `line`:`column`, each with
/// the text after it and the rows on screen.
fn typing(language: &str, text: &str, line: usize, column: usize) -> (SyntaxHighlighter, Vec<(InputEdit, Rope, Range<usize>)>) {
    let mut h = SyntaxHighlighter::new(language);
    let mut text = text.to_string();
    h.update(None, &Rope::from(text.as_str()), None);
    let at = rows_range(&text, line, 1).start + column;
    let mut edits = Vec::new();
    for i in 0..RUNS {
        let pos = at + i;
        text.insert(pos, 'x');
        let row = text[..pos].matches('\n').count();
        let column = pos - text[..pos].rfind('\n').map_or(0, |n| n + 1);
        let (start, end) = (Point { row, column }, Point { row, column: column + 1 });
        let edit = InputEdit {
            start_byte: pos,
            old_end_byte: pos,
            new_end_byte: pos + 1,
            start_position: start,
            old_end_position: start,
            new_end_position: end,
        };
        edits.push((edit, Rope::from(text.as_str()), rows_range(&text, line.saturating_sub(20), VISIBLE_ROWS)));
    }
    (h, edits)
}

/// A keystroke parsed on the spot, as the editor did before it parsed in the background.
fn keystrokes(language: &str, text: &str, line: usize, column: usize) -> (Duration, Duration) {
    let theme = syntax_theme(Appearance::Dark);
    let (mut h, edits) = typing(language, text, line, column);
    let mut next = edits.into_iter();
    time(|| {
        let (edit, rope, visible) = next.next().unwrap();
        h.update(Some(edit), &rope, None);
        black_box(h.styles(&visible, &*theme));
    })
}

/// What the UI thread does for a keystroke now: the edit moves the old colours, and the rows on
/// screen are styled from them. No parse.
fn keystrokes_on_ui(language: &str, text: &str, line: usize, column: usize) -> (Duration, Duration) {
    let theme = syntax_theme(Appearance::Dark);
    let (mut h, edits) = typing(language, text, line, column);
    let mut next = edits.into_iter();
    time(|| {
        let (edit, rope, visible) = next.next().unwrap();
        h.edit_tree(Some(edit), &rope);
        black_box(h.styles(&visible, &*theme));
    })
}

/// From a keystroke to its new colours: the edit, the background parse and its injections, taking
/// it, and the rows styled again. Run in a row here; the editor adds a hop to the background thread
/// and back, and the next frame.
fn colour_latency(language: &str, text: &str, line: usize, column: usize) -> (Duration, Duration) {
    let theme = syntax_theme(Appearance::Dark);
    let (mut h, edits) = typing(language, text, line, column);
    let mut next = edits.into_iter();
    time(|| {
        let (edit, rope, visible) = next.next().unwrap();
        h.edit_tree(Some(edit), &rope);
        let parsed = h.background_parse().unwrap().run().unwrap();
        assert!(h.apply_parsed(parsed));
        black_box(h.styles(&visible, &*theme));
    })
}

#[test]
#[ignore]
fn highlight_timings() {
    let theme = syntax_theme(Appearance::Dark);
    let rust = rust_file(10_000);
    println!("fixtures: rust 10000 lines ({} KB), diff 5000 rows, 20 code blocks of 25 lines", rust.len() / 1024);

    println!("\nthe editor (gpui-component's highlighter, as the editor drives it)");
    report(
        "10k-line Rust: first parse + visible styles",
        "< 50 ms, off the UI thread",
        time(|| {
            let mut h = SyntaxHighlighter::new("rust");
            h.update(None, &Rope::from(rust.as_str()), None);
            black_box(h.styles(&rows_range(&rust, 5_000, VISIBLE_ROWS), &*theme));
        }),
    );
    report("10k-line Rust: keystroke, UI thread (edit + visible styles)", "< 1 ms", keystrokes_on_ui("rust", &rust, 5_000, 4));
    report("  the same functions in modules of 50", "< 1 ms", keystrokes_on_ui("rust", &rust_in_modules(10_000), 5_001, 12));
    let short = rust_file(300);
    report("  a 300-line file", "< 1 ms", keystrokes_on_ui("rust", &short, 150, 4));
    report("10k-line Rust: colour latency (edit to new colours)", "(no target)", colour_latency("rust", &rust, 5_000, 4));
    report("  the same functions in modules of 50", "(no target)", colour_latency("rust", &rust_in_modules(10_000), 5_001, 12));
    report("  a 300-line file", "(no target)", colour_latency("rust", &short, 150, 4));
    println!("\nthe editor before it parsed in the background: each keystroke parsed on the spot");
    report("10k-line Rust: keystroke + visible styles", "", keystrokes("rust", &rust, 5_000, 4));
    // Line 1008, column 30 is inside a `format!` among the first 512 macros, which have layers, so
    // the edit parses that layer again.
    report("  typed inside a macro with a layer", "", keystrokes("rust", &rust, 1_008, 30));
    report("  the same file with no macros", "", keystrokes("rust", &rust.replace("format!", "format"), 5_000, 4));
    // Tree-sitter's incremental parse and changed ranges walk the root's children, so a flat file of
    // about 900 functions costs more than the same functions in modules.
    report("  the same functions in modules of 50", "", keystrokes("rust", &rust_in_modules(10_000), 5_001, 12));
    report("  10k-line Go, which has no injections", "", keystrokes("go", &go_file(10_000), 5_000, 4));
    let mut h = SyntaxHighlighter::new("rust");
    h.update(None, &Rope::from(rust.as_str()), None);
    report(
        "10k-line Rust: visible styles (a scroll step)",
        "< 0.5 ms",
        time(|| {
            black_box(h.styles(&rows_range(&rust, 7_000, VISIBLE_ROWS), &*theme));
        }),
    );

    println!("\ndiffs and code blocks (atelier_ui::syntax: background threads, one warm highlighter per language)");
    let lines = DiffLine::parse(&diff(5_000));
    let (old, new) = sides(&lines);
    compute("rust", "fn warm() {}", &theme);
    report(
        "5k-row diff: both sides parsed and styled",
        "< 50 ms, off the UI thread",
        time(|| {
            black_box((compute("rust", &old.text, &theme), compute("rust", &new.text, &theme)));
        }),
    );
    let (old_runs, new_runs) = (compute("rust", &old.text, &theme), compute("rust", &new.text, &theme));
    // A warm highlighter reuses what it can from its last text; its colours must still be a fresh one's.
    let fresh = atelier_ui::syntax::compute_with(&mut SyntaxHighlighter::new("rust"), &old.text, &theme);
    assert!(old_runs == fresh, "a warm highlighter coloured the old side unlike a fresh one");
    report(
        "5k-row diff: a frame on the UI thread",
        "< 0.5 ms",
        time(|| {
            // What FileDiff does each render: both sides, their keys, each row's side, then the rows' runs.
            let (old, new) = sides(&lines);
            black_box((Key::new("rust", &old.text, Appearance::Dark), Key::new("rust", &new.text, Appearance::Dark)));
            let map = row_sides(&lines);
            let rows: Vec<_> = map[2_000..2_000 + VISIBLE_ROWS]
                .iter()
                .map(|side| {
                    side.and_then(|(s, at)| match s {
                        Side::Old => old_runs.get(at).cloned(),
                        Side::New => new_runs.get(at).cloned(),
                    })
                })
                .collect();
            black_box(rows);
        }),
    );
    let blocks: Vec<String> = (0..20).map(|i| if i % 2 == 0 { rust_file(25) } else { ts_file(25) }).collect();
    compute("typescript", "const warm = 1;", &theme);
    report(
        "20 code blocks, all parsed and styled",
        "off the UI thread",
        time(|| {
            for (i, b) in blocks.iter().enumerate() {
                black_box(compute(if i % 2 == 0 { "rust" } else { "typescript" }, b, &theme));
            }
        }),
    );
    report("one 25-line block, warm highlighter", "", time(|| {
        black_box(compute("rust", &blocks[0], &theme));
    }));
    report("one 25-line block, new highlighter", "(what warm saves)", time(|| {
        black_box(atelier_ui::syntax::compute_with(&mut SyntaxHighlighter::new("rust"), &blocks[0], &theme));
    }));
    let ts = ts_file(3_000);
    report("3k-line TypeScript: parsed and styled", "", time(|| {
        black_box(compute("typescript", &ts, &theme));
    }));
    let zig = zig_file(2_000);
    compute("zig", "const a = 1;", &theme);
    report("2k-line Zig: parsed and styled", "", time(|| {
        black_box(compute("zig", &zig, &theme));
    }));
}
