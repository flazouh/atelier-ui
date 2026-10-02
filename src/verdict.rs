//! What the reader thinks of the whole pull request, after GitQuiet's `Verdict.tsx` and
//! `docs/spec/verdict.md`.
//!
//! Folded, with Approve standing beside the fold: an approval needs no words and is the common answer.
//! Opening it shows the box and the three verbs in the merge tones: Approve in `success`, Request
//! changes in `danger`, Comment plain. It names the commit it is about, "About f4a97b1, the last commit
//! on this branch", and the verdict goes with that commit. Request changes and Comment need words;
//! Approve does not. There is no Approve on one's own pull request. When the owner reports a refusal,
//! the words stay in the box.

mod helpers;
mod structs;
mod types;

pub use helpers::{about, enabled, hint, needs_words, offered, placeholder, summary};
pub use structs::VerdictBox;
pub use types::{Decision, Verb, VerdictEvent};

#[cfg(test)]
mod tests;
