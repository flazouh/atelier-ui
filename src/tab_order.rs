//! The tabs of the single view as data: which are open in what order, which is active, what closing and
//! moving do, and how they group by project. The tab bar draws what this decides.

mod helpers;
mod structs;

pub use helpers::{grouped, visual_order};
pub use structs::{TabGroup, TabOrder};

#[cfg(test)]
mod tests;
