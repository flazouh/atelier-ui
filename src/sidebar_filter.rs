//! Which sessions the sidebar lists: a filter by what the reader asks of the list (the sessions that wait for
//! them, the ones at work, the ones put away). Pure: the app narrows the projects with
//! [`narrow`] before it hands them to the sidebar.

mod helpers;
mod types;

pub use helpers::{describe, hidden_by, narrow};
pub use types::SessionFilter;

#[cfg(test)]
use crate::{
    session_status::SessionStatus,
    sidebar_model::{SessionData},
};

#[cfg(test)]
mod tests;
