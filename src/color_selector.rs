//! beui's ColorSelector (`components/motion/color-selector.tsx`): a labelled group of round swatches, one of
//! them chosen. Each swatch is a 28px round `bg-muted/60` disc (atelier's control size; the web's is 44) with a
//! `border-foreground/10` edge and a 14px dot of its colour. The choice is a ring 2px outside the disc, 2px thick, in the choice's colour at 65%; it
//! glides from swatch to swatch on `SPRING_LAYOUT` (the shared-layout animation) and takes on the new colour
//! as it goes. A press sinks the swatch to 94% on `SPRING_PRESS`. A swatch that cannot be chosen is at 40%.
//!
//! The group is a radio group: Tab reaches the chosen swatch (the first, when none is chosen), and the
//! arrow keys move the choice, wrapping at the ends. The swatch under keyboard focus has a 2px outline, 4px
//! outside the ring. Under Reduce Motion the ring jumps and a press does not sink.
//!
//! It takes plain data: a value, a colour and a label for each swatch.

mod helpers;
mod structs;
mod types;

pub use helpers::{sizes, step, tab_stop};
pub use structs::{ColorSelector, Swatch};
pub use types::ChangeHandler;

#[cfg(test)]
use types::PRESS_SCALE;

#[cfg(test)]
use gpui_kit::SharedString;

#[cfg(test)]
mod tests;
