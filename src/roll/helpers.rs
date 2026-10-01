use super::types::Kind;

/// The new content's offset from its place, in shares of its height (positive is below), and its opacity,
/// with the rise at `rise` (1 at the start, 0 at rest) and the opacity at progress `fade` (0 to 1).
pub fn entering(kind: Kind, rise: f32, fade: f32) -> (f32, f32) {
    let (from, _) = kind.enter_opacity();
    (kind.start() * rise, from + (1. - from) * fade)
}

/// The old content's offset (negative is above) and opacity at exit progress `t` (0 to 1).
pub fn leaving(kind: Kind, t: f32) -> (f32, f32) {
    (-kind.start() * t, 1. - (1. - kind.exit_opacity()) * t)
}
