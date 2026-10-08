//! A list of an agent's models that the reader sorts and picks the default of: each row has a grip to drag it by and a star that makes it the
//! default. The list is controlled: it reports the new order or the new default, and its owner draws it again.
mod helpers;
mod structs;
mod types;
pub use helpers::{moved, star_colour};
pub use structs::ModelList;
pub use types::ModelRow;
#[cfg(test)]
mod tests;
