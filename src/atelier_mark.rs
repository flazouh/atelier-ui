//! The mark of atelier: our white "A" on a rounded tile of the accent, a little lighter at the top. The accent is
//! terracotta unless the owner gives one, so a colour picked from the palette can colour the mark. The "A" is
//! `assets/atelier-mark.svg`, the shape tools/mac/make-icon.sh draws the app icon from.

mod consts;
mod impls;
mod structs;

pub use consts::PATH;
pub use structs::{AtelierMark, Tile};

#[cfg(test)]
mod tests;
