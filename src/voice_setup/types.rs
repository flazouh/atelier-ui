/// How far along the setup is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SetupPhase {
    /// The model is coming down: 0 to 1.
    Download(f32),
    /// The model is on disk and loads; there is no number to show.
    Prepare,
    Ready,
}
