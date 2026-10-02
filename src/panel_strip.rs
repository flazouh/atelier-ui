//! The side-by-side layout: panels as columns of a set width in a row that scrolls sideways. Only the
//! columns the viewport shows, and a margin, are built and laid out; the others keep their state in the
//! entities the app made for them. A scroll settles on a column's edge; a new column slides in with the
//! Layout spring, and closing one closes its gap the same way.

mod helpers;
mod impls;
mod structs;
mod types;

pub use helpers::group_header;
pub use types::GROUP_HEADER;
