/// A bar of a one-colour mini chart, in design pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiniBar {
    pub height: f32,
    /// The radius of the two top corners; the bottom corners are square.
    pub top_radius: f32,
}
