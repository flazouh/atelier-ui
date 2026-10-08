use std::ops::Range;

use gpui_kit::HighlightStyle;

/// The languages built in. Each is a `gpui-kit` feature in `Cargo.toml`, so this list and the build cannot
/// drift apart: [`LANGUAGES`] is checked against the grammars in a test. Kotlin is left out: gpui-component
/// 0.6.6's Kotlin grammar does not load, and falls back to plain text.
pub const LANGUAGES: &[&str] = &[
    "rust",
    "typescript",
    "tsx",
    "javascript",
    "json",
    "markdown",
    "go",
    "python",
    "toml",
    "yaml",
    "zig",
    "bash",
    "c",
    "cpp",
    "csharp",
    "css",
    "html",
    "java",
    "lua",
    "php",
    "ruby",
    "scala",
    "sql",
    "swift",
    "proto",
    "diff",
    "make",
    "cmake",
    "graphql",
    "svelte",
    "astro",
    "elixir",
];

/// How many highlighted texts the cache keeps. A streamed block makes a new text at each token, so an
/// unbounded cache would grow with every answer.
pub const MAX_ENTRIES: usize = 512;

/// One line's styles, each range measured from the line's start.
pub type LineRuns = Vec<(Range<usize>, HighlightStyle)>;

/// Which of a diff's two texts a row belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Old,
    New,
}
