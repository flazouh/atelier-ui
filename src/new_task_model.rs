//! The new task dialog as data: the draft, when it can be created, and what creating it makes.

mod helpers;
mod structs;
mod types;

pub use helpers::{create, next_key};
pub use structs::Draft;
pub use types::Submit;

#[cfg(test)]
mod tests;
