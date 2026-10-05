/// Where [`crate::icon::Assets`] serves the "A".
pub const PATH: &str = "atelier/mark.svg";

/// The "A"'s width over its height, from its view box.
pub(super) const ASPECT: f32 = 120. / 82.;

/// The "A"'s width over the tile's.
pub(super) const WIDTH: f32 = 0.62;

/// The tile's corner over its side: the rounded square of an app icon.
pub(super) const CORNER: f32 = 0.23;

/// The "A" and the terracotta accent.
pub(super) const LETTER: u32 = 0xFFFFFF;
pub(super) const ACCENT: u32 = 0xE0875A;

/// How far the tile lightens toward white at its top and darkens toward black at its foot.
pub(super) const LIGHTEN: f32 = 0.18;
pub(super) const DARKEN: f32 = 0.12;
