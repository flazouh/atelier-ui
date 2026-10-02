//! A pull request's commits as one line, after GitQuiet's `Commits.tsx`: "6, newest 2h ago", which opens
//! to the list. A pull request with 127 commits is one line here rather than 127 rows.

mod helpers;
mod structs;

pub use helpers::how_many;
pub use structs::{CommitData, CommitsSummary};

#[cfg(test)]
mod tests;
