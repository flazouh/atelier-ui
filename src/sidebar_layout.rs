//! The sidebar's layout, configured in one place. Everything that decides how the sidebar lists and draws its rows is a
//! field of [`SidebarLayout`]: how it lists (by project or by priority), which sessions (the filter), what a row
//! shows (the agent's icon, the project's badge, the time), and how much it shows before it folds. The sidebar reads
//! its rows and its head from this one value; the Settings page edits it; the settings file keeps it. A new knob is
//! a new field here, with its default, and nowhere else.

mod structs;
mod types;

pub use structs::SidebarLayout;
pub use types::{BadgeShow, EARLIER_CHOICES, EARLIER_SHOWN, FOLD_AFTER, FOLD_CHOICES};

#[cfg(test)]
use crate::{sidebar_filter::SessionFilter, sidebar_model::ListMode};

#[cfg(test)]
mod tests;
