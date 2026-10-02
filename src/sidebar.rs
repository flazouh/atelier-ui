//! The sidebar: every project and, under each, its sessions, as one virtual list. [`crate::sidebar_model`]
//! decides the rows, their order, the folds and where the keys go; this draws them.
//!
//! Up and down move, left folds a project or goes to it from one of its sessions, right unfolds or goes in,
//! Enter opens a session or folds a project. Motion: a new session enters with [`Entrance`](crate::entrance::Entrance); a session whose
//! row moved (one that now needs you moves to the top) slides to its place with the Layout spring. Both
//! stand still with Reduce Motion.

mod head;
mod helpers;
mod structs;
mod types;

pub use structs::Sidebar;
pub use types::SidebarEvent;

#[cfg(test)]
use crate::sidebar_model::ProjectData;

#[cfg(test)]
mod tests;
