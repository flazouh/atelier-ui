use gpui_kit::{Context, FocusHandle};

use crate::{
    motion::{Animated, FrameClock, Spring},
    update_button::{consts::FOLLOW_OMEGA, structs::UpdateMotion},
};

impl UpdateMotion {
    pub(in crate::update_button) fn new(fraction: f32, cx: &mut Context<Self>) -> Self {
        let focus: FocusHandle = cx.focus_handle();
        Self {
            fraction: Animated::new(Spring::critical(FOLLOW_OMEGA), fraction),
            hover: Animated::new(Spring::TINT, 0.),
            hovered: false,
            clock: FrameClock::default(),
            focus,
        }
    }

    /// Advances one frame toward the fraction and the hover. Returns true while either still moves.
    pub(in crate::update_button) fn advance(&mut self, fraction: f32, reduce: bool) -> bool {
        self.fraction.set_target(fraction);
        self.hover.set_target(if self.hovered { 1. } else { 0. });
        let dt = self.clock.tick();
        let moving = self.fraction.step(dt, reduce) | self.hover.step(dt, reduce);
        if !moving {
            self.clock.rest();
        }
        moving
    }
}
