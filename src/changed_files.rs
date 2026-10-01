//! The card after a turn that lists every file the agent changed, in TodoList's card language:
//!
//! - Header `h-11 px-3.5`: "3 files changed" and the total `+A −R` in the diff colours, then a small
//!   primary Review button. The counts swap with [`Morph`](crate::morph::Morph) while the turn still runs.
//! - One row per file: a file icon, the path with its folder muted and its name in ink, a status word for
//!   added, deleted and renamed files, and `+a −r`. Pressing a row reports that file; Review reports the
//!   first one. Review is off until the turn ends.
//! - A long list shows five rows and "Show 7 more". The rest open with `Reveal`, and "Show fewer" folds
//!   them away again. A fold never hides a single row.
//! - While the turn runs, each new file enters with [`EntranceList`](crate::entrance::EntranceList), above the fold and, once the list is
//!   open, below it too. Rows already there when the list opens come in with the reveal instead.
//! - [`ChangedFiles::collapsible`] starts with the header alone and a chevron in it: a press on the header
//!   opens the rows and folds them away again. Review stays in the header and does not open them.

mod helpers;
mod structs;
mod types;

pub use helpers::{first_path, fold, fold_label, header_text, split_path, totals};
pub use structs::{ChangedFile, ChangedFiles, Fold};
pub use types::{FOLD_AT, FileChange};

#[cfg(test)]
mod tests;
