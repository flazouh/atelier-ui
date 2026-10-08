use super::types::SetupPhase;

/// How full the bar is for `phase`, or `None` while there is no number.
pub fn fraction(phase: SetupPhase) -> Option<f32> {
    match phase {
        SetupPhase::Download(done) => Some(if done.is_nan() {
            0.
        } else {
            done.clamp(0., 1.)
        }),
        SetupPhase::Prepare => None,
        SetupPhase::Ready => Some(1.),
    }
}

/// The words, and the size at the right when there is one, for a model of `total_mb`.
pub fn copy(phase: SetupPhase, total_mb: f32) -> (&'static str, Option<String>) {
    match phase {
        SetupPhase::Download(_) => {
            let done = fraction(phase).unwrap_or(0.);
            (
                "Downloading speech model",
                Some(format!("{:.0} / {:.0} MB", done * total_mb, total_mb)),
            )
        }
        SetupPhase::Prepare => ("Getting ready", None),
        SetupPhase::Ready => ("Ready", None),
    }
}
