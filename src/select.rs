//! beui's MorphSelect (`components/motion/select-morph.tsx`) on gpui-base's headless `Select`, which owns
//! focus transfer and the open/close/confirm/cancel actions; this module owns which option is highlighted as
//! the arrow keys move through the list, and picks it on confirm. The motion is beui's; the look is atelier's:
//!
//! - One surface. The trigger is also the closed state of the panel: opening grows the trigger's own box to
//!   the panel's on a spring (0.5s, bounce 0.22) and closing shrinks it back, never detaching. The header
//!   row is the trigger's row; its chevron turns on the same spring.
//! - Options fade in (Motion's default tween, 0.3s) and rise 6px on a spring (stiffness 500, damping 25),
//!   0.08s after the open and 35ms apart. The web also blurs each option in; gpui has no blur, so it
//!   does not.
//! - The fill is the trigger's, stepping to the panel's (lifted by the elevation in the design preview) as it
//!   grows; the rows' pills come from the panel's own fill ([`crate::design_preview::row_tone`]).
//! - Reduce Motion: the panel is there at once.
//!
//! Options may carry a group ([`SelectOption::group`]): each group opens with a muted heading. The surface
//! grows upward, the same motion mirrored, when the window has no room below the trigger and more above
//! ([`crate::placement`]), or always with [`Select::upward`].
//!
//! [`Select::compact`] is the trigger PromptInput reuses for its model picker (`h-8`, `text-xs`, no chevron,
//! no shadow, a fixed panel width): a second, smaller trigger skin over the same morph and item list.

mod helpers;
mod structs;
mod types;

pub use helpers::{
    bind_keys, header_inset, list_height_of, opens_in, panel_height_of, spring_unit, surface_at,
};
pub(crate) use helpers::trigger_tone;
pub(crate) use helpers::monogram;
pub use structs::{Select, SelectOption};
pub use types::{OPTION_INSET, ROW_GAP, SelectHandler};

#[cfg(test)]
use helpers::{should_light, step_active, type_ahead};
#[cfg(test)]
use types::ITEM_HEIGHT;

#[cfg(test)]
use gpui_kit::SharedString;

#[cfg(test)]
mod tests;
