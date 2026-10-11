//! NavList: a list of places to go, in groups that fold. A sidebar that has many places shows the groups and the
//! rows of the open ones, so the reader sees a short list first and opens what they need.
//!
//! - A group is a head (a fold arrow, the group's name and how many rows it has) and its rows under it. A press on
//!   the head folds or opens it. A group with no name has no head and always shows its rows: the list is then flat,
//!   as search results are.
//! - A row is one place: its words, and a quiet note at its right end when it has one. The chosen row stands on the
//!   strong card tone; the others are muted and take the hover tone under the pointer.
//! - The owner holds what is open and what is chosen. The list says which head or row was pressed and keeps nothing.
//! - Borderless, as every part here. An opened group's rows fade in; under Reduce Motion they show at once.
mod structs;
pub use structs::{NavGroup, NavList, NavRow};

#[cfg(test)]
mod tests;
