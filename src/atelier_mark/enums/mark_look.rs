use crate::theme::Appearance;

/// How the mark is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkLook {
    /// A paper "A" on the ink tile, a thin light edge and a glow of the accent under it. For the dark.
    Halo,
    /// An ink "A" on a tile of the accent, a little lighter at the top. For the light.
    Terracotta,
}

impl MarkLook {
    /// The look that reads on a page of `appearance`.
    pub fn of(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Dark => Self::Halo,
            Appearance::Light => Self::Terracotta,
        }
    }
}
