//! A short label on hover: gpui-base's unstyled `Tooltip` popup, in
//! the page inverted (dark ink in light, cream in dark), as the primary button is: no border, and one
//! step of shadow so it lifts off a card.
//!
//! Hand [`Tooltip::text`] to GPUI's own `.tooltip(...)` on a stateful element; GPUI owns the delay and
//! the placement.

mod helpers;
mod structs;

pub use structs::Tooltip;
