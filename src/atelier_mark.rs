//! The mark of atelier: our "A" on a rounded tile. In the dark it is a paper "A" on the ink tile, with a thin light edge
//! and a soft glow of the accent under it (the Halo); in the light it is an ink "A" on a tile of the accent (the
//! Terracotta). The accent is terracotta unless the owner gives one, so a colour picked from the palette can colour the
//! mark. The "A" is `assets/atelier-mark.svg`, the shape tools/mac/make-icon.sh draws the app icon from.

mod consts;
mod enums;
mod impls;
mod structs;

pub use consts::PATH;
pub use enums::MarkLook;
pub use structs::{AtelierMark, Tile};

#[cfg(test)]
mod tests;
