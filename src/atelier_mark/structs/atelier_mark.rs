use gpui_kit::{Hsla, IntoElement};

#[derive(IntoElement)]
pub struct AtelierMark {
    pub(in super::super) size: f32,
    /// Terracotta when none is given.
    pub(in super::super) accent: Option<Hsla>,
}
