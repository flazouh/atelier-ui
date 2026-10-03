//! Menu: beui.dev's ContextMenu (`components/motion/context-menu.tsx`) as a panel. The owner keeps the
//! open flag and the place (a [`crate::popover::Popover`], which also closes it on Escape, on Tab and on a
//! press outside); the menu is what goes in the panel.
//!
//! - Panel: atelier's look, which differs by site ([`MenuLook`]): the popover fill, the popover shadow, no border,
//!   and the padding, corners and row sizes that the site's menu had before this part.
//! - Row: 14px text on 20; the padding across and down, the gap and the corner are the site's. Under the pointer or the
//!   keys a row is the active one, and one pill (6.5% of the ink, 10% of the danger tone for a destructive
//!   row) glides to it on `Spring::LAYOUT`. A disabled row is at 40% and cannot be reached.
//! - Keys: Up and Down walk the rows and wrap round, Home and End go to the ends, Enter and Space choose,
//!   and typing letters jumps to the row whose words start with them (the letters clear after 500ms).
//!   The first row has focus when the menu opens.
//! - Open: with an [`Origin`] the panel unfolds from that point in 300ms: a clip that grows from a 16px
//!   square around the origin to the whole panel, with the fill fading in. Without one, or under Reduce
//!   Motion, it appears at once.
//! - Submenus: a row made with [`MenuItem::submenu`] opens a menu of its own beside it on hover, a press or
//!   Right; Left and Escape close it. [`entries_of`] builds a whole tree of them from [`Branch`]es.
//!
//! What gpui cannot draw is left out: the check's draw-on when a row toggles (the menu closes on choice, so
//! it is rarely seen).

mod helpers;
mod structs;
mod types;

pub use helpers::{collapsed, entries_of, fill_opacity, lead_icon, height, height_in, height_of, jump, panel_size, unfolded, walk};
pub use structs::{Inset, Menu, MenuItem, MenuLook};
pub use types::{BORDER, Branch, Choice, Entry, LINE, Lead, Origin, Pick, SLOT, Select, TEXT, Tone};

#[cfg(test)]
use gpui_kit::Pixels;

#[cfg(test)]
mod tests;
