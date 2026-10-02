//! Text whose letters each take their own color while their places stay put.
//!
//! `StyledText` with highlights shapes each differently colored stretch on its own: GPUI ends a font run wherever
//! the color changes, so kerning is lost at every color edge, and letters shift when the colors move, as a glimmer's
//! do every frame. A [`GlyphText`] shapes its text once, in its one text color, and paints each glyph where that
//! shaping put it, in the color its highlights and its [`Ink`] give it. It wraps like `StyledText` at the width it is
//! given, so the two lay out the same.
mod structs;
mod types;
pub use structs::GlyphText;
pub use types::Ink;
#[cfg(test)]
mod tests;
