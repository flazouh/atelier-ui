/// beui's `maxHeight` for the code viewport.
pub(super) const MAX_HEIGHT: f32 = 280.;

/// One row of code; the gutter and the tint bands count rows of this height.
pub(super) const LINE_HEIGHT: f32 = 20.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeBlockStatus {
    Streaming,
    Complete,
}
