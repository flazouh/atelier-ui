//! The image files a project offers as its icon, most likely first: a project's own mark (`logo`, `icon`, `favicon`,
//! `mark`, `brand`) in a place it tends to live (the root, `.github`, `public`, `assets`, `static`, `docs`, `brand`)
//! ahead of everything else, so a vendored icon set does not bury it. Ties break on depth, then name, so the order is
//! total and a reload shows the same list.

mod helpers;
mod types;

pub use helpers::{filter, is_icon_file, rank, rank_of};

#[cfg(test)]
mod tests;
