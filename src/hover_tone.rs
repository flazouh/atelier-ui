//! The hover tone of a field that opens a picker: it eases in on the pointer, holds while its picker is open, and
//! eases out, on the same spring as the buttons' hover tint. The owner keeps one per field, calls [`HoverTone::sync`]
//! each render, and asks for the level to draw.
use crate::motion::{Channel, Curve, Spring};

#[derive(Clone, Debug)]
pub struct HoverTone {
    tint: Channel,
    hovered: bool,
}
impl Default for HoverTone {
    fn default() -> Self {
        Self::new()
    }
}
impl HoverTone {
    pub fn new() -> Self {
        Self { tint: Channel::new(0.), hovered: false }
    }
    /// The pointer is over the field, or not.
    pub fn set_hovered(&mut self, hovered: bool) {
        self.hovered = hovered;
    }
    /// Eases toward the level the pointer and the open picker call for. Call every render.
    pub fn sync(&mut self, held: bool, reduce: bool) {
        let want = if self.hovered || held { 1. } else { 0. };
        if self.tint.target() != want {
            self.tint.animate(want, Curve::Spring(Spring::TINT), 0., reduce);
        }
    }
    /// 0 at rest, 1 hovered or held.
    pub fn level(&self) -> f32 {
        self.tint.value()
    }
    pub fn is_moving(&self) -> bool {
        self.tint.is_running()
    }
}
#[cfg(test)]
mod tests;
