//! beui's RangeSlider (`components/motion/range-slider.tsx`, with `lib/hooks/use-slider.ts` for the number):
//! a 40px track with `rounded-lg` and the `muted` fill, a `foreground/15` fill behind a 4x24px handle, and a
//! dot 4px across at each step. The handle sits 8px inside the track and the fill starts 8px past its left edge,
//! as `InlineSlider` does. Both follow the value through a spring (`SPRING_GLIDE`: stiff and critically damped, so
//! they follow a drag and never rebound off an end); grabbing the handle stretches it to 135% of its height on a
//! bouncy spring. A press anywhere on the track moves the handle there and starts a drag that goes on outside
//! the track. Arrow keys step, Page keys step ten, Home and End go to the ends. Disabled is 50%.
//!
//! The number is snapped to the step grid; a range the step does not divide (0 to 10 by 4) also allows its
//! maximum, so a drag near the end does not fall back a whole step ([`snap`]).

mod helpers;
mod structs;
mod types;

pub use helpers::{geometry, key_value, percent, snap, ticks};
pub use structs::RangeSlider;
pub use types::ChangeHandler;

#[cfg(test)]
mod tests;
