use gpui_kit::Hsla;

/// `color` at `t` of its strength.
pub(super) fn fade(color: Hsla, t: f32) -> Hsla {
    Hsla {
        a: color.a * t,
        ..color
    }
}
