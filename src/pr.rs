//! A pull request as plain data, and the words and marks every PR part shares: [`crate::pr_card::PrCard`]
//! above the composer, [`crate::pr_chip::PrChip`] inline in agent text, and later the PR view. atelier-ui never
//! fetches any of it; the app hands it over.

mod structs;
mod types;

pub use structs::{Checks, PrChipData, PrFacts};
pub use types::{ChecksSummary, PrState, ReviewState};

#[cfg(test)]
mod tests;
