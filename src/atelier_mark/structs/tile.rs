use gpui_kit::Hsla;

/// The colours of the mark, worked out from its look and its accent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tile {
    /// The tile at its top and at its foot.
    pub top: Hsla,
    pub bottom: Hsla,
    /// The "A".
    pub ink: Hsla,
    /// A thin line round the tile, when it has one.
    pub edge: Option<Hsla>,
    /// The glow that rises from the foot, when there is one.
    pub glow: Option<Hsla>,
}
