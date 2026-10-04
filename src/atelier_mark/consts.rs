/// Where [`crate::icon::Assets`] serves the "A".
pub const PATH: &str = "atelier/mark.svg";

/// The "A"'s width over its height, from its view box.
pub(super) const ASPECT: f32 = 120. / 82.;

/// The "A"'s width over the tile's.
pub(super) const WIDTH: f32 = 0.62;

/// The tile's corner over its side: the rounded square of an app icon.
pub(super) const CORNER: f32 = 0.23;

/// The brand's colours: the ink and the paper of the tile and the "A", and the terracotta accent.
pub(super) const INK: u32 = 0x141413;
pub(super) const PAPER: u32 = 0xF0EEE6;
pub(super) const ACCENT: u32 = 0xE0875A;

/// How far the Terracotta tile lightens toward white at its top and darkens toward black at its foot.
pub(super) const LIGHTEN: f32 = 0.18;
pub(super) const DARKEN: f32 = 0.12;

/// The Halo's edge and glow: the edge's opacity, the glow's opacity at the foot, and how much of the tile the glow rises.
pub(super) const EDGE_ALPHA: f32 = 0.14;
pub(super) const GLOW_ALPHA: f32 = 0.5;
pub(super) const GLOW_RISE: f32 = 0.45;
