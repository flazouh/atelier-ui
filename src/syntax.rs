//! Syntax colours for code outside the editor: diffs, code blocks and the pull request view, drawn with the
//! editor's own highlighter (gpui-component's tree-sitter `SyntaxHighlighter`) and the editor's colours
//! (the theme in force), so a line looks the same in all of them.
//!
//! - A text is parsed as a whole, never line by line, so a string or a comment across lines comes out
//!   right. A diff parses each of its two sides as one text and maps each row to its side's line.
//! - Each text is parsed once. Its styles, split into lines, are cached by language, content and theme
//!   ([`SyntaxCache`]), so scrolling, hover and a re-render never parse again.
//! - Every text parses on a background thread, never on the UI thread: even a short one can cost more
//!   than a frame when it needs a query built ("Performance" in `docs/code-editor.md`).
//! - Each background thread keeps one highlighter per language: building one compiles its queries,
//!   which costs far more than parsing a code block.
//! - While a text parses, its slot (a code block, a diff's side) keeps the colours of its last text on
//!   each row that did not change, so a streamed block does not flash plain at each token. A new row
//!   draws plain until the parse lands. The colours change nothing else, so the rows do not move.
//! - A slot's new text drops the parse of its old one: the background job checks the slot's
//!   generation before it starts and before it writes, and stops when a newer text replaced it.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    hash::{DefaultHasher, Hash, Hasher},
    ops::Range,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use gpui_kit::{
    App, AppContext, ElementId, Global, HighlightStyle, SharedString,
    base::input::Rope,
    component::highlighter::{HighlightTheme, SyntaxHighlighter},
};

use crate::{
    file_diff::{DiffLine, DiffLineKind},
    theme::Appearance,
};

/// The languages built in. Each is a `gpui-kit` feature in `Cargo.toml`, so this list and the build cannot
/// drift apart: [`LANGUAGES`] is checked against the grammars in a test. Kotlin is left out: gpui-component
/// 0.6.6's Kotlin grammar does not load, and falls back to plain text.
pub const LANGUAGES: &[&str] = &[
    "rust", "typescript", "tsx", "javascript", "json", "markdown", "go", "python", "toml", "yaml", "zig", "bash",
    "c", "cpp", "csharp", "css", "html", "java", "lua", "php", "ruby", "scala", "sql", "swift", "proto",
    "diff", "make", "cmake", "graphql", "svelte", "astro", "elixir",
];

/// How many highlighted texts the cache keeps. A streamed block makes a new text at each token, so an
/// unbounded cache would grow with every answer.
pub const MAX_ENTRIES: usize = 512;

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

/// One line's styles, each range measured from the line's start.
pub type LineRuns = Vec<(Range<usize>, HighlightStyle)>;

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

/// Which of a diff's two texts a row belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Old,
    New,
}

/// One side of a diff as a whole text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SideText {
    pub text: String,
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

/// What a cached entry is for.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    language: SharedString,
    hash: u64,
    appearance: Appearance,
}

impl Key {
    pub fn new(language: &str, text: &str, appearance: Appearance) -> Self {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        Self { language: SharedString::from(language.to_string()), hash: hasher.finish(), appearance }
    }
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

/// A slot's last highlighted text.
struct Shown {
    key: Key,
    text: Arc<str>,
}

/// Highlighted texts, by language, content and theme. One per app.
#[derive(Default)]
pub struct SyntaxCache {
    entries: HashMap<Key, Arc<Vec<LineRuns>>>,
    /// The entries' keys, oldest first.
    order: VecDeque<Key>,
    pending: HashSet<Key>,
    /// Each slot's last text that had its colours, kept as long as its entry.
    shown: HashMap<ElementId, Shown>,
    /// Each slot's generation: it goes up with each text the slot asks to parse. At most
    /// [`MAX_ENTRIES`] slots, the oldest forgotten first.
    generations: HashMap<ElementId, Arc<AtomicU64>>,
    /// The slots in `generations`, oldest first.
    slots: VecDeque<ElementId>,
    /// How many background parses ran, for tests.
    pub parsed: Arc<AtomicUsize>,
    /// How many texts were parsed, for tests and for the bench.
    pub computed: usize,
    /// Time spent in [`highlight`] on the UI thread since the owner last reset it, for a frame log.
    pub spent: Duration,
}

impl Global for SyntaxCache {}

impl SyntaxCache {
    /// The entry for `key`, computing it with `compute` only the first time.
    pub fn get_or_compute(&mut self, key: Key, compute: impl FnOnce() -> Vec<LineRuns>) -> Arc<Vec<LineRuns>> {
        if let Some(hit) = self.entries.get(&key) {
            return hit.clone();
        }
        self.computed += 1;
        let lines = Arc::new(compute());
        if self.order.len() == MAX_ENTRIES
            && let Some(oldest) = self.order.pop_front()
        {
            self.entries.remove(&oldest);
            self.shown.retain(|_, shown| shown.key != oldest);
        }
        self.order.push_back(key.clone());
        self.entries.insert(key, lines.clone());
        lines
    }

    /// `slot`'s generation, a new one for a slot not seen lately.
    fn generation(&mut self, slot: ElementId) -> Arc<AtomicU64> {
        if let Some(generation) = self.generations.get(&slot) {
            return generation.clone();
        }
        if self.slots.len() == MAX_ENTRIES
            && let Some(oldest) = self.slots.pop_front()
        {
            self.generations.remove(&oldest);
            self.shown.remove(&oldest);
        }
        self.slots.push_back(slot.clone());
        self.generations.entry(slot).or_default().clone()
    }

    fn get(&self, key: &Key) -> Option<Arc<Vec<LineRuns>>> {
        self.entries.get(key).cloned()
    }
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

#[cfg(test)]
mod tests;
