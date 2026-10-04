use gpui_kit::{Hsla, IntoElement};

use super::super::enums::MarkLook;

#[derive(IntoElement)]
pub struct AtelierMark {
    pub(in super::super) size: f32,
    /// The theme's own look when none is asked for.
    pub(in super::super) look: Option<MarkLook>,
    /// Terracotta when none is given.
    pub(in super::super) accent: Option<Hsla>,
}
