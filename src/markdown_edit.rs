//! The Markdown a comment box's toolbar writes: bold, italic, code and link around the chosen words, and
//! a quote or a list mark on every chosen line. Pure, so each edit is tested on a string. It returns the
//! new text and what to choose after, so the words stay chosen and a second press can act on them.

mod helpers;
mod types;

pub use helpers::apply;
pub use types::Format;

#[cfg(test)]
mod tests;
