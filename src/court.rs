//! Pull requests sorted by who owes the next move, after GitQuiet's working set: its four Courts, Needs
//! You, Waiting, Running and Settled, with its words (`CONTEXT.md`, `domain/sittings.ts`).
//!
//! atelier-ui does not decide which Court a pull request sits in; the app does, from its own reading of the
//! pull request, and hands it over as data. atelier-ui files them: Courts in reading order, an empty Court left
//! out, a pull request listed twice kept once in its most urgent Court, and inside each Court the newest
//! change first.

mod helpers;
mod structs;
mod types;

pub use helpers::{checks_text, courts};
pub use structs::{CourtItem, CourtList};
pub use types::Court;

#[cfg(test)]
use types::TITLE_LEAST;

#[cfg(test)]
use crate::pr::Checks;

#[cfg(test)]
mod tests;
