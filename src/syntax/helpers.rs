use std::{
    cell::RefCell,
    collections::HashMap,
    ops::Range,
    sync::{Arc, atomic::Ordering},
    time::Instant,
};

use gpui_kit::{
    App,
    AppContext,
    ElementId,
    HighlightStyle,
    base::input::Rope,
    component::highlighter::{HighlightTheme, SyntaxHighlighter},
};

use crate::file_diff::{DiffLine, DiffLineKind};
use super::structs::{Key, Shown, SideText, SyntaxCache};
use super::types::{LineRuns, Side};

/// The language to highlight a path as: by its whole name first (`Makefile`), then its extension. A
/// trailing `:line` is ignored. `None` leaves it plain.
pub fn language_for(path: &str) -> Option<&'static str> {
    let path = crate::file_icon::strip_location(path);
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    match name {
        "Makefile" | "makefile" | "GNUmakefile" => return Some("make"),
        "CMakeLists.txt" => return Some("cmake"),
        _ => {}
    }
    let extension = name.rsplit_once('.')?.1.to_lowercase();
    Some(match extension.as_str() {
        "rs" => "rust",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" | "jsx" => "tsx",
        "js" | "mjs" | "cjs" => "javascript",
        "json" | "jsonc" => "json",
        "md" | "markdown" => "markdown",
        "go" => "go",
        "py" | "pyi" => "python",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "zig" | "zon" => "zig",
        "sh" | "bash" | "zsh" => "bash",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
        "cs" => "csharp",
        "css" => "css",
        "html" | "htm" => "html",
        "java" => "java",
        "lua" => "lua",
        "php" => "php",
        "rb" | "rake" => "ruby",
        "scala" | "sc" => "scala",
        "sql" => "sql",
        "swift" => "swift",
        "proto" => "proto",
        "diff" | "patch" => "diff",
        "mk" => "make",
        "cmake" => "cmake",
        "graphql" | "gql" => "graphql",
        "svelte" => "svelte",
        "astro" => "astro",
        "ex" | "exs" => "elixir",
        _ => return None,
    })
}

thread_local! {
    /// This thread's highlighters, one per language.
    static HIGHLIGHTERS: RefCell<HashMap<String, SyntaxHighlighter>> = RefCell::new(HashMap::new());
}

/// `text` highlighted as a whole in `language`, then split into lines, with this thread's highlighter
/// for it (built the first time).
pub fn compute(language: &str, text: &str, theme: &HighlightTheme) -> Vec<LineRuns> {
    HIGHLIGHTERS.with(|h| {
        let mut h = h.borrow_mut();
        let highlighter = h.entry(language.to_string()).or_insert_with(|| SyntaxHighlighter::new(language));
        compute_with(highlighter, text, theme)
    })
}

/// `text` highlighted with `highlighter`, then split into lines. With no edit given, the highlighter
/// parses `text` whole, whatever it held before.
pub fn compute_with(highlighter: &mut SyntaxHighlighter, text: &str, theme: &HighlightTheme) -> Vec<LineRuns> {
    highlighter.update(None, &Rope::from(text), None);
    let styles = highlighter.styles(&(0..text.len()), theme);
    split_lines(text, &styles)
}

/// Whole-text styles, cut at each line and moved to start from it.
fn split_lines(text: &str, styles: &[(Range<usize>, HighlightStyle)]) -> Vec<LineRuns> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut at = 0;
    for line in text.split('\n') {
        let end = start + line.len();
        // Styles are in text order, so each line picks up from where the last one stopped.
        while at < styles.len() && styles[at].0.end <= start {
            at += 1;
        }
        let mut runs = LineRuns::new();
        for (range, style) in styles[at..].iter().take_while(|(r, _)| r.start < end) {
            let (s, e) = (range.start.max(start), range.end.min(end));
            if s < e && *style != HighlightStyle::default() {
                runs.push((s - start..e - start, *style));
            }
        }
        out.push(runs);
        start = end + 1;
    }
    out
}

/// The two texts a diff's rows make: the old side (context and removed rows) and the new side (context
/// and added rows).
pub fn sides(lines: &[DiffLine]) -> (SideText, SideText) {
    let join = |keep: fn(DiffLineKind) -> bool| {
        lines.iter().filter(|l| keep(l.kind)).map(|l| l.text.as_ref()).collect::<Vec<_>>().join("\n")
    };
    (
        SideText { text: join(|k| matches!(k, DiffLineKind::Context | DiffLineKind::Removed)) },
        SideText { text: join(|k| matches!(k, DiffLineKind::Context | DiffLineKind::Added)) },
    )
}

/// For each row, its side and its line there. A context row reads from the new side; a hunk header is
/// not code.
pub fn row_sides(lines: &[DiffLine]) -> Vec<Option<(Side, usize)>> {
    let (mut old, mut new) = (0, 0);
    lines
        .iter()
        .map(|l| match l.kind {
            DiffLineKind::Hunk => None,
            DiffLineKind::Removed => {
                old += 1;
                Some((Side::Old, old - 1))
            }
            DiffLineKind::Added => {
                new += 1;
                Some((Side::New, new - 1))
            }
            DiffLineKind::Context => {
                old += 1;
                new += 1;
                Some((Side::New, new - 1))
            }
        })
        .collect()
}

/// `text`'s lines with the colours of `last`'s on each line that is the same in both, and none on the
/// others: what a slot shows while `text` parses.
pub fn carry(last_text: &str, last: &[LineRuns], text: &str) -> Vec<LineRuns> {
    let old: Vec<&str> = last_text.split('\n').collect();
    text.split('\n')
        .enumerate()
        .map(|(i, line)| match (old.get(i), last.get(i)) {
            (Some(was), Some(runs)) if *was == line => runs.clone(),
            _ => LineRuns::new(),
        })
        .collect()
}

/// The editor's syntax colours for the theme in force.
fn theme_in(cx: &App) -> Arc<HighlightTheme> {
    gpui_kit::component::Theme::global(cx).highlight_theme.clone()
}

/// `text`'s lines in `language`, highlighted, from the cache. A text not cached yet starts a parse on a
/// background thread, and the windows redraw when it lands. Until then `slot` keeps its last text's
/// colours on the lines that did not change; with no last text, this returns `None`.
pub fn highlight(language: &str, text: &str, slot: ElementId, cx: &mut App) -> Option<Arc<Vec<LineRuns>>> {
    let start = Instant::now();
    let lines = lookup(language, text, slot, cx);
    cx.global_mut::<SyntaxCache>().spent += start.elapsed();
    lines
}

fn lookup(language: &str, text: &str, slot: ElementId, cx: &mut App) -> Option<Arc<Vec<LineRuns>>> {
    let appearance = crate::theme::ActiveTheme::theme(cx).appearance;
    let key = Key::new(language, text, appearance);
    if !cx.has_global::<SyntaxCache>() {
        cx.set_global(SyntaxCache::default());
    }
    let cache = cx.global_mut::<SyntaxCache>();
    if let Some(hit) = cache.get(&key) {
        if cache.shown.get(&slot).is_none_or(|shown| shown.key != key) {
            cache.shown.insert(slot, Shown { key, text: text.into() });
        }
        return Some(hit);
    }
    let last = cache
        .shown
        .get(&slot)
        .filter(|shown| shown.key.language == key.language && shown.key.appearance == appearance)
        .and_then(|shown| Some((shown.text.clone(), cache.get(&shown.key)?)));
    if cache.pending.insert(key.clone()) {
        let generation = cache.generation(slot);
        let mine = generation.fetch_add(1, Ordering::Relaxed) + 1;
        let current = move || generation.load(Ordering::Relaxed) == mine;
        let parsed = cache.parsed.clone();
        let theme = theme_in(cx);
        let (language, owned) = (language.to_string(), text.to_string());
        let parse = cx.background_spawn({
            let current = current.clone();
            async move {
                current().then(|| {
                    parsed.fetch_add(1, Ordering::Relaxed);
                    compute(&language, &owned, &theme)
                })
            }
        });
        cx.spawn(async move |cx| {
            let lines = parse.await;
            cx.update(|cx| {
                let cache = cx.global_mut::<SyntaxCache>();
                cache.pending.remove(&key);
                if let Some(lines) = lines.filter(|_| current()) {
                    cache.get_or_compute(key, || lines);
                }
                // A dropped parse redraws too, so another slot that waited on the same text asks again.
                cx.refresh_windows();
            });
        })
        .detach();
    }
    last.map(|(last_text, last)| Arc::new(carry(&last_text, &last, text)))
}
