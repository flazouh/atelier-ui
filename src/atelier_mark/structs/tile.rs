use gpui_kit::Hsla;

/// The colours of the mark, worked out from its accent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tile {
    /// The tile at its top and at its foot.
    pub top: Hsla,
    pub bottom: Hsla,
    /// The "A".
    pub letter: Hsla,
}
